use super::models::Method;
use std::sync::LazyLock;

static CRON_MARK: &[u8] = b"[cron]";
static CRON_FINDER: LazyLock<memchr::memmem::Finder<'static>> =
    LazyLock::new(|| memchr::memmem::Finder::new(CRON_MARK));

#[derive(Clone, Debug)]
pub enum LineKind<'a> {
    Empty,
    Http {
        method: Method,
        path: &'a [u8],
        status: u16,
        duration_ms: f32,
        hour: Option<u8>,
    },
    Cron {
        event: u8, // 0=start, 1=done, 2=fail
        name: &'a [u8],
        duration_ms: Option<f32>,
    },
    Unmatched(&'a [u8]),
}

#[inline]
fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

pub fn skip_ansi(buf: &[u8], mut i: usize, end: usize) -> usize {
    while i + 1 < end && buf[i] == 0x1b && buf[i + 1] == b'[' {
        i += 2;
        while i < end {
            let c = buf[i];
            i += 1;
            if (0x40..=0x7e).contains(&c) {
                break;
            }
        }
    }
    i
}

pub fn skip_space_ansi(buf: &[u8], mut i: usize, end: usize) -> usize {
    loop {
        i = skip_ansi(buf, i, end);
        if i >= end {
            return i;
        }
        let c = buf[i];
        if c == b' ' || c == b'\t' {
            i += 1;
            continue;
        }
        return i;
    }
}

fn only_space_ansi_left(buf: &[u8], i: usize, end: usize) -> bool {
    skip_space_ansi(buf, i, end) >= end
}

#[inline(always)]
fn skip_timestamp(buf: &[u8], start: usize, end: usize) -> Option<(usize, Option<u8>)> {
    if end - start < 20 {
        return None;
    }
    let a = start;
    if buf[a + 4] == b'-'
        && buf[a + 7] == b'-'
        && (buf[a + 10] == b'T' || buf[a + 10] == b' ')
        && buf[a + 13] == b':'
        && buf[a + 16] == b':'
        && buf[a + 19] == b':'
    {
        let h1 = buf[a + 11];
        let h2 = buf[a + 12];
        let hour = if h1.is_ascii_digit() && h2.is_ascii_digit() {
            Some((h1 - b'0') * 10 + (h2 - b'0'))
        } else {
            None
        };
        Some((skip_space_ansi(buf, a + 20, end), hour))
    } else {
        None
    }
}

fn parse_method(buf: &[u8], mut i: usize, end: usize) -> Option<(Method, usize)> {
    i = skip_space_ansi(buf, i, end);
    if i + 3 <= end && &buf[i..i + 3] == b"GET" {
        let after = i + 3;
        let next = if after < end { buf[after] } else { b' ' };
        if next == b' ' || next == b'\t' || next == 0x1b || after >= end {
            return Some((Method::Get, after));
        }
    }
    if i + 4 <= end && &buf[i..i + 4] == b"POST" {
        let after = i + 4;
        let next = if after < end { buf[after] } else { b' ' };
        if next == b' ' || next == b'\t' || next == 0x1b || after >= end {
            return Some((Method::Post, after));
        }
    }
    const OTHER_METHODS: &[(Method, &[u8])] = &[
        (Method::Put, b"PUT"),
        (Method::Patch, b"PATCH"),
        (Method::Delete, b"DELETE"),
        (Method::Options, b"OPTIONS"),
        (Method::Head, b"HEAD"),
    ];
    for &(m, kw) in OTHER_METHODS {
        let klen = kw.len();
        if i + klen <= end && &buf[i..i + klen] == kw {
            let after = i + klen;
            let next = if after < end { buf[after] } else { b' ' };
            if next == b' ' || next == b'\t' || next == 0x1b || after >= end {
                return Some((m, after));
            }
        }
    }
    None
}

fn parse_u16_fast(buf: &[u8], mut i: usize, end: usize) -> Option<(u16, usize)> {
    i = skip_space_ansi(buf, i, end);
    let mut val: u16 = 0;
    let mut count = 0;
    while i < end {
        i = skip_ansi(buf, i, end);
        if i >= end {
            break;
        }
        let c = buf[i];
        if is_digit(c) {
            val = val * 10 + (c - b'0') as u16;
            count += 1;
            i += 1;
        } else {
            break;
        }
    }
    if count > 0 {
        Some((val, i))
    } else {
        None
    }
}

fn parse_f32_fast(buf: &[u8], mut i: usize, end: usize) -> Option<(f32, usize)> {
    i = skip_space_ansi(buf, i, end);
    let mut int_part: u32 = 0;
    let mut count = 0;
    while i < end {
        i = skip_ansi(buf, i, end);
        if i >= end {
            break;
        }
        let c = buf[i];
        if is_digit(c) {
            int_part = int_part * 10 + (c - b'0') as u32;
            count += 1;
            i += 1;
        } else {
            break;
        }
    }
    if count == 0 {
        return None;
    }

    let mut frac_part: f32 = 0.0;
    let mut divisor: f32 = 1.0;
    if i < end && buf[i] == b'.' {
        i += 1;
        while i < end {
            i = skip_ansi(buf, i, end);
            if i >= end {
                break;
            }
            let c = buf[i];
            if is_digit(c) {
                divisor *= 10.0;
                frac_part += (c - b'0') as f32 / divisor;
                i += 1;
            } else {
                break;
            }
        }
    }

    Some((int_part as f32 + frac_part, i))
}

fn parse_cron_line<'a>(buf: &'a [u8], i: usize, _end: usize) -> Option<LineKind<'a>> {
    let lower_buf: Vec<u8> = buf[i.._end].iter().map(|b| b.to_ascii_lowercase()).collect();
    let lower_len = lower_buf.len();

    let is_started = memchr::memmem::find(&lower_buf, b"started").is_some()
        || memchr::memmem::find(&lower_buf, b"start").is_some();
    let is_completed = memchr::memmem::find(&lower_buf, b"completed").is_some()
        || memchr::memmem::find(&lower_buf, b"done").is_some()
        || memchr::memmem::find(&lower_buf, b"success").is_some();
    let is_failed = memchr::memmem::find(&lower_buf, b"failed").is_some()
        || memchr::memmem::find(&lower_buf, b"fail").is_some()
        || memchr::memmem::find(&lower_buf, b"error").is_some();

    if !is_started && !is_completed && !is_failed {
        return None;
    }

    let cron_idx = CRON_FINDER.find(&lower_buf)?;
    let after_cron = cron_idx + 6;

    let mut name_start = after_cron;
    while name_start < lower_len && (lower_buf[name_start] == b' ' || lower_buf[name_start] == b'\t') {
        name_start += 1;
    }

    let first_word_end = {
        let mut e = name_start;
        while e < lower_len && lower_buf[e] != b' ' && lower_buf[e] != b'\t' && lower_buf[e] != b'\r' && lower_buf[e] != b'\n' {
            e += 1;
        }
        e
    };
    let first_word = &lower_buf[name_start..first_word_end];
    if first_word == b"done"
        || first_word == b"started"
        || first_word == b"completed"
        || first_word == b"failed"
        || first_word == b"start"
        || first_word == b"fail"
    {
        name_start = first_word_end;
        while name_start < lower_len && (lower_buf[name_start] == b' ' || lower_buf[name_start] == b'\t') {
            name_start += 1;
        }
    }

    let mut name_end = name_start;
    while name_end < lower_len
        && lower_buf[name_end] != b' '
        && lower_buf[name_end] != b'\t'
        && lower_buf[name_end] != b'\r'
        && lower_buf[name_end] != b'\n'
    {
        name_end += 1;
    }

    if name_start >= name_end {
        return None;
    }

    let name_bytes = &buf[i + name_start..i + name_end];

    let mut duration_ms = None;
    if let Some(in_idx) = memchr::memmem::find(&lower_buf, b"in ") {
        let after_in = in_idx + 3;
        let mut d_end = after_in;
        while d_end < lower_len && lower_buf[d_end].is_ascii_digit() {
            d_end += 1;
        }
        if d_end > after_in {
            if let Ok(s) = std::str::from_utf8(&lower_buf[after_in..d_end]) {
                if let Ok(val) = s.parse::<f32>() {
                    duration_ms = Some(val);
                }
            }
        }
    } else if let Some(after_idx) = memchr::memmem::find(&lower_buf, b"after ") {
        let after_after = after_idx + 6;
        let mut d_end = after_after;
        while d_end < lower_len && lower_buf[d_end].is_ascii_digit() {
            d_end += 1;
        }
        if d_end > after_after {
            if let Ok(s) = std::str::from_utf8(&lower_buf[after_after..d_end]) {
                if let Ok(val) = s.parse::<f32>() {
                    duration_ms = Some(val);
                }
            }
        }
    }

    let event = if is_failed {
        2
    } else if is_completed {
        1
    } else {
        0
    };

    Some(LineKind::Cron {
        event,
        name: name_bytes,
        duration_ms,
    })
}

pub fn parse_line_bytes(line: &[u8]) -> LineKind<'_> {
    let i = skip_space_ansi(line, 0, line.len());
    if i >= line.len() {
        return LineKind::Empty;
    }

    let mut prefix_dur: Option<f32> = None;
    let (cur_i, hour) = if let Some((after_ts, h)) = skip_timestamp(line, i, line.len()) {
        (after_ts, h)
    } else if let Some((dur, after_dur)) = parse_f32_fast(line, i, line.len()) {
        let post_dur = skip_space_ansi(line, after_dur, line.len());
        if post_dur + 1 < line.len()
            && (line[post_dur] == b'm' || line[post_dur] == b'M')
            && (line[post_dur + 1] == b's' || line[post_dur + 1] == b'S')
        {
            prefix_dur = Some(dur);
            (skip_space_ansi(line, post_dur + 2, line.len()), None)
        } else {
            (i, None)
        }
    } else {
        (i, None)
    };

    if let Some(cron) = parse_cron_line(line, cur_i, line.len()) {
        return cron;
    }

    let (method, after_m) = match parse_method(line, cur_i, line.len()) {
        Some(res) => res,
        None => return LineKind::Unmatched(line),
    };

    let p_start = skip_space_ansi(line, after_m, line.len());
    let mut p_end = p_start;
    while p_end < line.len() {
        let c = line[p_end];
        if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' || c == 0x1b {
            break;
        }
        p_end += 1;
    }
    if p_start >= p_end {
        return LineKind::Unmatched(line);
    }
    let path = &line[p_start..p_end];

    let (status, after_s) = match parse_u16_fast(line, p_end, line.len()) {
        Some(res) => res,
        None => (200, p_end),
    };

    let (dur, after_dur) = match parse_f32_fast(line, after_s, line.len()) {
        Some(res) => res,
        None => (prefix_dur.unwrap_or(0.0), after_s),
    };

    let post_dur = skip_space_ansi(line, after_dur, line.len());
    let (clean_dur, final_i) = if prefix_dur.is_some() {
        (prefix_dur.unwrap(), after_s)
    } else if post_dur < line.len()
        && (line[post_dur] == b'm' || line[post_dur] == b'M')
        && post_dur + 1 < line.len()
        && (line[post_dur + 1] == b's' || line[post_dur + 1] == b'S')
    {
        (dur, post_dur + 2)
    } else if post_dur < line.len()
        && (line[post_dur] == b's' || line[post_dur] == b'S')
    {
        (dur * 1000.0, post_dur + 1)
    } else {
        (dur, after_dur)
    };

    let rest = skip_space_ansi(line, final_i, line.len());
    let rest_valid = rest >= line.len() || line[rest] == b'-' || line[rest] == b'/' || is_digit(line[rest]);

    if !rest_valid {
        return LineKind::Unmatched(line);
    }

    LineKind::Http {
        method,
        path,
        status,
        duration_ms: clean_dur,
        hour,
    }
}
