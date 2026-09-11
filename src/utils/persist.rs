//! localStorage helpers via `document::eval` (WebView2 origin storage).

use dioxus::prelude::*;

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

pub async fn load_item(key: &str) -> Option<String> {
    let script = format!(
        "try {{ return localStorage.getItem({}); }} catch (e) {{ return null; }}",
        json_string(key)
    );
    document::eval(&script)
        .join::<Option<String>>()
        .await
        .ok()
        .flatten()
}

pub fn set_item(key: &str, value: &str) {
    let script = format!(
        "try {{ localStorage.setItem({}, {}); }} catch (e) {{}}",
        json_string(key),
        json_string(value)
    );
    let _ = document::eval(&script);
}

/// Toggle the `dark` class on `<html>` (matches the reference app).
pub fn set_html_dark(dark: bool) {
    let script = if dark {
        "document.documentElement.classList.add('dark');"
    } else {
        "document.documentElement.classList.remove('dark');"
    };
    let _ = document::eval(script);
}
