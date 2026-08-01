use super::models::PathNormMode;
use std::borrow::Cow;

pub fn normalize_path_bytes<'a>(path: &'a [u8], mode: PathNormMode) -> Cow<'a, [u8]> {
    match mode {
        PathNormMode::Raw => Cow::Borrowed(path),
        PathNormMode::StripQuery => strip_query(path),
        PathNormMode::CollapseIds => collapse_ids(path),
    }
}

#[allow(dead_code)]
pub fn normalize_path<'a>(path: &'a str, mode: PathNormMode) -> Cow<'a, str> {
    match normalize_path_bytes(path.as_bytes(), mode) {
        Cow::Borrowed(b) => {
            if let Ok(s) = std::str::from_utf8(b) {
                Cow::Borrowed(s)
            } else {
                Cow::Owned(String::from_utf8_lossy(b).into_owned())
            }
        }
        Cow::Owned(vec) => Cow::Owned(String::from_utf8_lossy(&vec).into_owned()),
    }
}

fn strip_query<'a>(path: &'a [u8]) -> Cow<'a, [u8]> {
    if let Some(idx) = path.iter().position(|&c| c == b'?' || c == b'#') {
        Cow::Borrowed(&path[..idx])
    } else {
        Cow::Borrowed(path)
    }
}

fn collapse_ids<'a>(path: &'a [u8]) -> Cow<'a, [u8]> {
    let p = match strip_query(path) {
        Cow::Borrowed(s) => s,
        Cow::Owned(v) => return Cow::Owned(collapse_segments_owned(&v)),
    };

    let mut start = 0usize;
    let mut needs_collapse = false;
    for i in 0..=p.len() {
        if i == p.len() || p[i] == b'/' {
            let seg = &p[start..i];
            if collapse_segment(seg) != seg {
                needs_collapse = true;
                break;
            }
            start = i + 1;
        }
    }

    if !needs_collapse {
        return Cow::Borrowed(p);
    }

    let mut out = Vec::with_capacity(p.len());
    start = 0;
    for i in 0..=p.len() {
        if i == p.len() || p[i] == b'/' {
            let seg = &p[start..i];
            out.extend_from_slice(collapse_segment(seg));
            if i < p.len() {
                out.push(b'/');
            }
            start = i + 1;
        }
    }
    Cow::Owned(out)
}

fn collapse_segments_owned(p: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(p.len());
    let mut start = 0usize;
    for i in 0..=p.len() {
        if i == p.len() || p[i] == b'/' {
            let seg = &p[start..i];
            out.extend_from_slice(collapse_segment(seg));
            if i < p.len() {
                out.push(b'/');
            }
            start = i + 1;
        }
    }
    out
}

#[inline]
fn collapse_segment(seg: &[u8]) -> &[u8] {
    if seg.is_empty() {
        return seg;
    }
    if is_object_id(seg) || is_long_numeric(seg) || is_uuid(seg) || is_pr_id(seg) || is_code_id(seg) {
        b":id"
    } else {
        seg
    }
}

#[inline]
fn is_object_id(seg: &[u8]) -> bool {
    seg.len() == 24 && seg.iter().all(u8::is_ascii_hexdigit)
}

#[inline]
fn is_long_numeric(seg: &[u8]) -> bool {
    seg.len() >= 6 && seg.iter().all(u8::is_ascii_digit)
}

#[inline]
fn is_uuid(seg: &[u8]) -> bool {
    if seg.len() != 36 {
        return false;
    }
    if seg[8] != b'-' || seg[13] != b'-' || seg[18] != b'-' || seg[23] != b'-' {
        return false;
    }
    for (i, &c) in seg.iter().enumerate() {
        if i != 8 && i != 13 && i != 18 && i != 23 && !c.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

#[inline]
fn is_pr_id(seg: &[u8]) -> bool {
    if seg.len() < 12 || !seg[..3].eq_ignore_ascii_case(b"PR-") {
        return false;
    }
    let rest = &seg[3..];
    let Some(dash) = rest.iter().position(|&c| c == b'-') else {
        return false;
    };
    let letters = &rest[..dash];
    let digits = &rest[dash + 1..];
    letters.len() >= 3
        && letters.iter().all(u8::is_ascii_alphabetic)
        && digits.len() >= 8
        && digits.iter().all(u8::is_ascii_digit)
}

#[inline]
fn is_code_id(seg: &[u8]) -> bool {
    let Some(d1) = seg.iter().position(|&c| c == b'-') else {
        return false;
    };
    let a = &seg[..d1];
    let rest = &seg[d1 + 1..];
    let Some(d2) = rest.iter().position(|&c| c == b'-') else {
        return false;
    };
    let b = &rest[..d2];
    let digits = &rest[d2 + 1..];
    a.len() >= 2
        && a.iter().all(u8::is_ascii_alphabetic)
        && b.len() >= 2
        && b.iter().all(u8::is_ascii_alphabetic)
        && digits.len() >= 6
        && digits.iter().all(u8::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_object_id() {
        let p = b"/api/users/507f1f77bcf86cd799439011/profile";
        assert_eq!(
            normalize_path_bytes(p, PathNormMode::CollapseIds).as_ref(),
            b"/api/users/:id/profile"
        );
    }

    #[test]
    fn strip_query() {
        let out = normalize_path_bytes(b"/api/x?foo=1&bar=2", PathNormMode::StripQuery);
        assert_eq!(out.as_ref(), b"/api/x");
        assert!(matches!(out, Cow::Borrowed(_)));
    }

    #[test]
    fn collapse_noop_borrows() {
        let p = b"/api/v1/health";
        let out = normalize_path_bytes(p, PathNormMode::CollapseIds);
        assert_eq!(out.as_ref(), p);
        assert!(matches!(out, Cow::Borrowed(_)));
    }

    #[test]
    fn collapse_long_numeric_only() {
        let p1 = b"/api/v1/users/2"; // short number, preserved
        assert_eq!(
            normalize_path_bytes(p1, PathNormMode::CollapseIds).as_ref(),
            b"/api/v1/users/2"
        );

        let p2 = b"/api/v1/users/1234567"; // >=6 digits, collapsed
        assert_eq!(
            normalize_path_bytes(p2, PathNormMode::CollapseIds).as_ref(),
            b"/api/v1/users/:id"
        );
    }
}
