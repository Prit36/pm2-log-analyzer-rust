//! Log format classifier — port of `wasm/zip-core/src/lib.rs`.
//!
//! Classifies incoming log files and archive entries into PM2 (API logs),
//! MongoDB logs, skipped files (hidden/metadata/error logs), or unknown
//! (which fallback to the active tab).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogKind {
    Pm2,
    Mongo,
    Unknown,
    Skip,
}

impl LogKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogKind::Pm2 => "pm2",
            LogKind::Mongo => "mongo",
            LogKind::Unknown => "unknown",
            LogKind::Skip => "skip",
        }
    }
}

/// Classifies a log entry by its file name or path.
///
/// Returns `Some(LogKind)` if the name deterministically matches a category,
/// or `None` if content inspection is required.
pub fn classify_by_name(name: &str) -> Option<LogKind> {
    let lower = name.to_ascii_lowercase().replace('\\', "/");
    let file_name = lower.rsplit('/').next().unwrap_or(&lower);

    // Skip hidden files, system files, OSX metadata
    if file_name.starts_with('.') || file_name.starts_with("__macosx") {
        return Some(LogKind::Skip);
    }

    // Skip error logs — they do not contain API timing metrics
    if file_name.contains("error") {
        return Some(LogKind::Skip);
    }

    // Mongo patterns: mongod.log*, mongodb.log*, mongo*.log*
    if file_name.starts_with("mongod")
        || file_name.starts_with("mongodb")
        || file_name.starts_with("mongo.")
        || file_name.starts_with("mongo-")
        || file_name.starts_with("mongo_")
        || file_name.contains("mongod.log")
        || file_name.contains("mongodb.log")
    {
        return Some(LogKind::Mongo);
    }

    // API / PM2 patterns: api-out.log*, pm2*, out.log, etc.
    if file_name.contains("api-out")
        || file_name.contains("api_out")
        || file_name.starts_with("api.")
        || file_name.starts_with("api-")
        || file_name.contains("pm2")
        || file_name.starts_with("out.log")
    {
        return Some(LogKind::Pm2);
    }

    None
}

/// Classifies a log entry by inspecting the first 4KB of raw content.
pub fn classify_by_content(data: &[u8]) -> LogKind {
    let sample_len = data.len().min(4096);
    let sample = &data[..sample_len];

    // Check for Mongo JSON or legacy formats
    if sample.windows(8).any(|w| w == b"\"$date\"")
        || sample.windows(5).any(|w| w == b"\"msg\"")
        || sample.windows(5).any(|w| w == b"\"ctx\"")
        || sample.windows(15).any(|w| w == b"[initandlisten]")
        || sample.windows(6).any(|w| w == b"[conn")
    {
        return LogKind::Mongo;
    }

    // Check for HTTP / PM2 log lines
    if sample.windows(4).any(|w| w == b"GET " || w == b"POST")
        || sample.windows(4).any(|w| w == b"PUT " || w == b"HEAD")
        || sample.windows(7).any(|w| w == b"DELETE " || w == b"OPTIONS")
        || sample.windows(6).any(|w| w == b"[cron]")
        || sample.windows(5).any(|w| w == b"[PM2]")
    {
        return LogKind::Pm2;
    }

    LogKind::Unknown
}

/// Classify a log by name first, falling back to content inspection if unknown.
pub fn classify_log(name: &str, sample: &[u8]) -> LogKind {
    if let Some(kind) = classify_by_name(name) {
        if kind != LogKind::Unknown {
            return kind;
        }
    }
    classify_by_content(sample)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_by_name() {
        assert_eq!(classify_by_name("mongod.log.1"), Some(LogKind::Mongo));
        assert_eq!(classify_by_name("mongod.log.10.gz"), Some(LogKind::Mongo));
        assert_eq!(classify_by_name("mongodb.log"), Some(LogKind::Mongo));
        assert_eq!(classify_by_name("api-out.log.1"), Some(LogKind::Pm2));
        assert_eq!(classify_by_name("api-error.log.5.gz"), Some(LogKind::Skip));
        assert_eq!(classify_by_name("error.log"), Some(LogKind::Skip));
        assert_eq!(classify_by_name(".DS_Store"), Some(LogKind::Skip));
        assert_eq!(classify_by_name("__MACOSX/._log.txt"), Some(LogKind::Skip));
        assert_eq!(classify_by_name("custom-service.log"), None);
    }

    #[test]
    fn test_classify_by_content() {
        let mongo_json = br#"{"t":{"$date":"2026-07-24T12:00:00.000Z"},"s":"I","c":"COMMAND","msg":"Slow query"}"#;
        assert_eq!(classify_by_content(mongo_json), LogKind::Mongo);

        let pm2_http = b"2026-07-24T00:00:01: GET /api/users 200 12.3 ms - 45\n";
        assert_eq!(classify_by_content(pm2_http), LogKind::Pm2);

        let cron_log = b"2026-07-24T00:00:01: [cron] sync-job started\n";
        assert_eq!(classify_by_content(cron_log), LogKind::Pm2);

        let unknown = b"Hello world, random logging line\n";
        assert_eq!(classify_by_content(unknown), LogKind::Unknown);
    }
}
