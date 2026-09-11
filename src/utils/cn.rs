//! `cn` helper — joins conditional class fragments (mirrors `src/utils/cn.ts`).

pub fn cn(parts: &[&str]) -> String {
    let mut out = String::new();
    for part in parts {
        if part.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(part);
    }
    out
}
