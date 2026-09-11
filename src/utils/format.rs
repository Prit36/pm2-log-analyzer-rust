//! Formatting helpers — exact ports of `src/utils/format.ts` (JS semantics,
//! including `toFixed` rounding which differs from Rust's `{:.n}`).

/// JS `Number.prototype.toFixed(digits)` rounding (closest decimal, ties to larger).
fn js_to_fixed(value: f64, digits: usize) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    let negative = value < 0.0;
    let v = value.abs();
    // 25 guard decimals expose the distance to a decimal tie for every finite f64.
    let precise = format!("{:.*}", digits + 25, v);
    let (int_part, frac_part) = match precise.split_once('.') {
        Some((i, f)) => (i.to_string(), f.to_string()),
        None => (precise.clone(), String::new()),
    };

    let mut out_int = int_part;
    let mut out_frac = frac_part;
    if digits == 0 {
        let round_up = out_frac.as_bytes().first().is_some_and(|c| *c >= b'5');
        if round_up {
            out_int = increment_decimal(&out_int);
        }
        let text = out_int;
        return if negative { format!("-{text}") } else { text };
    }

    if out_frac.len() <= digits {
        while out_frac.len() < digits {
            out_frac.push('0');
        }
        let text = format!("{out_int}.{out_frac}");
        return if negative { format!("-{text}") } else { text };
    }
    let keep: String = out_frac.chars().take(digits).collect();
    let next = out_frac.as_bytes()[digits];
    if next >= b'5' {
        let bumped = increment_decimal(&format!("{out_int}{keep}"));
        // Re-split after carry (the integer part may have grown by one digit).
        let split = bumped.len() - digits;
        let text = format!("{}.{}", &bumped[..split], &bumped[split..]);
        return if negative { format!("-{text}") } else { text };
    }
    let text = format!("{out_int}.{keep}");
    if negative { format!("-{text}") } else { text }
}

fn increment_decimal(digits: &str) -> String {
    let mut bytes: Vec<u8> = digits.bytes().collect();
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        if bytes[i] == b'9' {
            bytes[i] = b'0';
        } else {
            bytes[i] += 1;
            return String::from_utf8(bytes).unwrap_or_else(|_| digits.to_string());
        }
    }
    let mut out = String::from("1");
    out.push_str(&String::from_utf8(bytes).unwrap_or_default());
    out
}

/// `formatMs` from the reference: `>= 1000` → seconds with 2 decimals, else integer ms.
pub fn format_ms(ms: f64) -> String {
    if !ms.is_finite() {
        return "-".to_string();
    }
    if ms >= 1000.0 {
        format!("{}s", js_to_fixed(ms / 1000.0, 2))
    } else {
        format!("{}ms", js_to_fixed(ms, 0))
    }
}

/// `formatNum`: `Intl.NumberFormat()` grouping for the default en-US locale.
pub fn format_num(n: u64) -> String {
    group_thousands(&n.to_string())
}

pub fn format_num_i64(n: i64) -> String {
    if n < 0 {
        format!("-{}", group_thousands(&n.unsigned_abs().to_string()))
    } else {
        group_thousands(&n.to_string())
    }
}

fn group_thousands(digits: &str) -> String {
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let len = digits.len();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// `formatBytes`.
pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const K: f64 = 1024.0;
    let sizes = ["B", "KB", "MB", "GB"];
    let i = ((bytes as f64).ln() / K.ln()).floor() as usize;
    let i = i.min(sizes.len() - 1);
    let value = bytes as f64 / K.powi(i as i32);
    let fixed = js_to_fixed(value, 1);
    let trimmed = fixed.trim_end_matches(".0").to_string();
    format!("{trimmed} {}", sizes[i])
}

/// `formatDate`: `new Date(y, m-1, d).toLocaleDateString()` — en-GB (system locale here).
pub fn format_date(date: Option<&str>) -> String {
    let Some(date) = date else {
        return String::new();
    };
    let mut parts = date.split('-');
    let (Some(y), Some(m), Some(d)) = (parts.next(), parts.next(), parts.next()) else {
        return date.to_string();
    };
    let Ok(year) = y.parse::<i32>() else {
        return date.to_string();
    };
    let Ok(month) = m.parse::<u32>() else {
        return date.to_string();
    };
    let day_str = &d[..d.len().min(2)];
    let Ok(day) = day_str.parse::<u32>() else {
        return date.to_string();
    };
    format!("{day:02}/{month:02}/{year}")
}

/// `formatDateTime`: `new Date(ts).toLocaleString()` — en-GB (system locale here).
pub fn format_date_time(ts: Option<&str>) -> String {
    use chrono::{Local, TimeZone};
    let Some(ts) = ts else {
        return String::new();
    };
    let parsed = chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.with_timezone(&Local))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
                .map(|ndt| Local.from_local_datetime(&ndt).earliest().unwrap_or_default())
                .map_err(|_| ())
        });
    match parsed {
        Ok(dt) => format!(
            "{}, {}:{}:{}",
            dt.format("%d/%m/%Y"),
            dt.format("%H"),
            dt.format("%M"),
            dt.format("%S")
        ),
        Err(_) => ts.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_fixed_matches_js_ties() {
        assert_eq!(js_to_fixed(0.5, 0), "1");
        assert_eq!(js_to_fixed(1.5, 0), "2");
        assert_eq!(js_to_fixed(2.5, 0), "3");
        assert_eq!(js_to_fixed(0.125, 2), "0.13");
        assert_eq!(js_to_fixed(12.5, 0), "13");
        assert_eq!(js_to_fixed(0.4449999999999999, 2), "0.44");
        assert_eq!(js_to_fixed(2.6749999999999994, 2), "2.67");
        assert_eq!(js_to_fixed(1.0049999999999999, 2), "1.00");
        assert_eq!(js_to_fixed(1.2649999856948853, 0), "1");
        assert_eq!(js_to_fixed(1234.5678, 2), "1234.57");
    }

    #[test]
    fn formats() {
        assert_eq!(format_ms(999.4), "999ms");
        assert_eq!(format_ms(1000.0), "1.00s");
        assert_eq!(format_ms(1234.0), "1.23s");
        assert_eq!(format_num(1234567), "1,234,567");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1 MB");
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_date(Some("2026-07-24")), "24/07/2026");
    }
}
