use super::models::Method;
use memchr::memchr;
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
fn skip_timestamp(buf: &[u8], start: usize, end: usize) -> Option<usize> {
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
        Some(skip_space_ansi(buf, a + 20, end))
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
        (Method::Head, b"HEAD"),
        (Method::Patch, b"PATCH"),
        (Method::Delete, b"DELETE"),
        (Method::Options, b"OPTIONS"),
    ];
    for &(method, bytes) in OTHER_METHODS {
        if i + bytes.len() > end {
            continue;
        }
        if &buf[i..i + bytes.len()] != bytes {
            continue;
        }
        let after = i + bytes.len();
        let next = if after < end { buf[after] } else { b' ' };
        if next == b' ' || next == b'\t' || next == 0x1b || after >= end {
            return Some((method, after));
        }
    }
    None
}

fn read_token(buf: &[u8], mut i: usize, end: usize) -> Option<(usize, usize, usize)> {
    i = skip_space_ansi(buf, i, end);
    if i >= end {
        return None;
    }
    let start = i;
    while i < end {
        let c = buf[i];
        if c == b' ' || c == b'\t' || c == 0x1b {
            break;
        }
        i += 1;
    }
    if i == start {
        return None;
    }
    Some((start, i, i))
}

#[inline(always)]
fn parse_float(buf: &[u8], mut i: usize, end: usize) -> Option<(f32, usize)> {
    i = skip_space_ansi(buf, i, end);
    if i >= end {
        return None;
    }
    let mut val: u32 = 0;
    let mut digits = 0;
    while i < end {
        let c = buf[i];
        if c >= b'0' && c <= b'9' {
            val = val * 10 + (c - b'0') as u32;
            digits += 1;
            i += 1;
        } else {
            break;
        }
    }
    if digits == 0 {
        return None;
    }
    if i < end && buf[i] == b'.' {
        i += 1;
        let mut frac: u32 = 0;
        let mut frac_digits = 0;
        while i < end {
            let c = buf[i];
            if c >= b'0' && c <= b'9' {
                frac = frac * 10 + (c - b'0') as u32;
                frac_digits += 1;
                i += 1;
            } else {
                break;
            }
        }
        static DIVS: [f32; 6] = [1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0];
        let divisor = if (frac_digits as usize) < DIVS.len() {
            DIVS[frac_digits as usize]
        } else {
            10.0f32.powi(frac_digits as i32)
        };
        let res = (val as f32) + (frac as f32) / divisor;
        Some((res, i))
    } else {
        Some((val as f32, i))
    }
}


fn has_non_space(buf: &[u8], start: usize, end: usize) -> bool {
    let mut i = start;
    while i < end {
        let c = buf[i];
        if c > 32 && c != 0x1b {
            return true;
        }
        if c == 0x1b {
            i = skip_ansi(buf, i, end);
            continue;
        }
        i += 1;
    }
    false
}

fn try_http_a<'a>(buf: &'a [u8], start: usize, end: usize) -> Option<LineKind<'a>> {
    let mut i = skip_space_ansi(buf, start, end);
    if let Some(ni) = skip_timestamp(buf, i, end) {
        i = ni;
    }
    let (method, ni) = parse_method(buf, i, end)?;
    i = ni;
    let (ps, pe, ni) = read_token(buf, i, end)?;
    i = ni;
    let (ss, se, ni) = read_token(buf, i, end)?;
    if se - ss != 3 {
        return None;
    }
    let s0 = buf[ss];
    let s1 = buf[ss + 1];
    let s2 = buf[ss + 2];
    if !is_digit(s0) || !is_digit(s1) || !is_digit(s2) {
        return None;
    }
    let status = ((s0 - b'0') as u16) * 100 + ((s1 - b'0') as u16) * 10 + ((s2 - b'0') as u16);
    i = ni;
    let (dur, ni) = parse_float(buf, i, end)?;
    i = skip_space_ansi(buf, ni, end);
    if i + 1 >= end || buf[i] != b'm' || buf[i + 1] != b's' {
        return None;
    }
    i = skip_space_ansi(buf, i + 2, end);
    if i >= end || buf[i] != b'-' {
        return None;
    }
    i = skip_space_ansi(buf, i + 1, end);
    if i >= end {
        return None;
    }
    if buf[i] == b'-' {
        i += 1;
    } else {
        let b0 = i;
        while i < end && is_digit(buf[i]) {
            i += 1;
        }
        if i == b0 {
            return None;
        }
    }
    if !only_space_ansi_left(buf, i, end) {
        return None;
    }
    Some(LineKind::Http {
        method,
        path: &buf[ps..pe],
        status,
        duration_ms: dur,
    })
}

fn try_http_b<'a>(buf: &'a [u8], start: usize, end: usize) -> Option<LineKind<'a>> {
    let mut i = skip_space_ansi(buf, start, end);
    let (dur, ni) = parse_float(buf, i, end)?;
    i = skip_space_ansi(buf, ni, end);
    if i + 1 >= end || buf[i] != b'm' || buf[i + 1] != b's' {
        return None;
    }
    i = skip_space_ansi(buf, i + 2, end);
    let (method, ni) = parse_method(buf, i, end)?;
    i = ni;
    let (ps, pe, ni) = read_token(buf, i, end)?;
    if !only_space_ansi_left(buf, ni, end) {
        return None;
    }
    Some(LineKind::Http {
        method,
        path: &buf[ps..pe],
        status: 200,
        duration_ms: dur,
    })
}

fn find_cron_mark(buf: &[u8], from: usize, end: usize) -> Option<usize> {
    if from >= end {
        return None;
    }
    if memchr(b'[', &buf[from..end]).is_none() {
        return None;
    }
    CRON_FINDER.find(&buf[from..end]).map(|rel| from + rel)
}

fn try_cron<'a>(buf: &'a [u8], start: usize, end: usize) -> Option<LineKind<'a>> {
    let mut i = skip_space_ansi(buf, start, end);
    if let Some(ni) = skip_timestamp(buf, i, end) {
        i = ni;
    }
    let cron_idx = find_cron_mark(buf, i, end)?;
    let mut k = i;
    while k < cron_idx {
        k = skip_ansi(buf, k, end);
        if k >= cron_idx {
            break;
        }
        let c = buf[k];
        if c == b' ' || c == b'\t' {
            k += 1;
            continue;
        }
        return None;
    }
    i = skip_space_ansi(buf, cron_idx + 6, end);
    let event = if i + 5 <= end && &buf[i..i + 5] == b"start" && (i + 5 >= end || buf[i + 5] == b' ')
    {
        i += 5;
        0u8
    } else if i + 4 <= end && &buf[i..i + 4] == b"done" && (i + 4 >= end || buf[i + 4] == b' ') {
        i += 4;
        1
    } else if i + 4 <= end && &buf[i..i + 4] == b"fail" && (i + 4 >= end || buf[i + 4] == b' ') {
        i += 4;
        2
    } else {
        return None;
    };
    i = skip_space_ansi(buf, i, end);
    let name_slice = &buf[i..end];
    let mut lo = 0usize;
    let mut hi = name_slice.len();
    while lo < hi && (name_slice[lo] == b' ' || name_slice[lo] == b'\t') {
        lo += 1;
    }
    while hi > lo && (name_slice[hi - 1] == b' ' || name_slice[hi - 1] == b'\t') {
        hi -= 1;
    }
    let trimmed_name = &name_slice[lo..hi];
    if trimmed_name.is_empty() {
        return None;
    }

    let mut duration_ms = None;
    let mut final_name = trimmed_name;
    if trimmed_name.ends_with(b"ms") {
        let body = &trimmed_name[..trimmed_name.len() - 2];
        let mut body_end = body.len();
        while body_end > 0 && (body[body_end - 1] == b' ' || body[body_end - 1] == b'\t') {
            body_end -= 1;
        }
        let trimmed_body = &body[..body_end];
        if let Some(sp) = trimmed_body.iter().rposition(|&c| c == b' ' || c == b'\t') {
            let num = &trimmed_body[sp + 1..];
            let name_part = &trimmed_body[..sp];
            if !name_part.is_empty() {
                if let Some((v, consumed)) = parse_float(num, 0, num.len()) {
                    if consumed == num.len() {
                        final_name = name_part;
                        duration_ms = Some(v);
                    }
                }
            }
        }
    }

    Some(LineKind::Cron {
        event,
        name: final_name,
        duration_ms,
    })
}

pub fn parse_line_bytes<'a>(buf: &'a [u8], start: usize, mut end: usize) -> LineKind<'a> {
    if end > start && buf[end - 1] == b'\r' {
        end -= 1;
    }
    if end <= start {
        return LineKind::Empty;
    }

    // Fast Path 1: Try HTTP Format A first (matches >95% of requests instantly)
    if let Some(k) = try_http_a(buf, start, end) {
        return k;
    }

    // Fast Path 2: Try HTTP Format B
    if let Some(k) = try_http_b(buf, start, end) {
        return k;
    }

    // Fallback: Check if empty or whitespace only
    if !has_non_space(buf, start, end) {
        return LineKind::Empty;
    }

    // Fallback: Check for Cron events
    if find_cron_mark(buf, start, end).is_some() {
        if let Some(k) = try_cron(buf, start, end) {
            return k;
        }
    }

    LineKind::Unmatched(&buf[start..end])
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_a() {
        let s = b"2026-07-24T00:00:10: GET /api/health 200 12.5 ms - 42";
        match parse_line_bytes(s, 0, s.len()) {
            LineKind::Http {
                method,
                path,
                status,
                duration_ms,
            } => {
                assert_eq!(method, Method::Get);
                assert_eq!(path, b"/api/health");
                assert_eq!(status, 200);
                assert!((duration_ms - 12.5).abs() < 0.01);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn http_a_ansi() {
        let s = b"\x1b[0mPOST /api/admin/dashboard \x1b[32m200\x1b[0m 71.197 ms - 223\x1b[0m";
        match parse_line_bytes(s, 0, s.len()) {
            LineKind::Http {
                method,
                path,
                status,
                duration_ms,
            } => {
                assert_eq!(method, Method::Post);
                assert_eq!(path, b"/api/admin/dashboard");
                assert_eq!(status, 200);
                assert!((duration_ms - 71.197).abs() < 0.01);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn http_b() {
        let s = b"68064.174ms\tPOST /api/admin/user/getuserbyrole";
        match parse_line_bytes(s, 0, s.len()) {
            LineKind::Http {
                method,
                path,
                duration_ms,
                ..
            } => {
                assert_eq!(method, Method::Post);
                assert_eq!(path, b"/api/admin/user/getuserbyrole");
                assert!((duration_ms - 68064.174).abs() < 0.01);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
