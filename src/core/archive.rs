//! Archive decompression and extraction — port of `zipExtractor.ts` and `zip-core`.
//!
//! Handles extraction of `.zip` and `.gz` archives, nested gzip entries,
//! filename and content-based classification, and returns memory-backed
//! `LoadedSource` items ready for the PM2 and MongoDB parsers.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use flate2::read::GzDecoder;

use crate::core::classify::{classify_log, LogKind};
use crate::core::pm2::LoadedSource;

/// Represents an archive extraction outcome partitioned by log kind.
#[derive(Default, Debug, Clone)]
pub struct ExtractedArchive {
    pub pm2_sources: Vec<LoadedSource>,
    pub mongo_sources: Vec<LoadedSource>,
    pub unknown_sources: Vec<LoadedSource>,
    pub skipped: Vec<String>,
    pub duration_ms: u64,
}

impl ExtractedArchive {
    pub fn is_empty(&self) -> bool {
        self.pm2_sources.is_empty()
            && self.mongo_sources.is_empty()
            && self.unknown_sources.is_empty()
    }

    pub fn total_files(&self) -> usize {
        self.pm2_sources.len() + self.mongo_sources.len() + self.unknown_sources.len()
    }
}

/// Checks if a file path represents a supported archive (.zip or .gz).
pub fn is_archive(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    is_archive_name(&name)
}

/// Checks if a file name represents a supported archive.
pub fn is_archive_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".zip") || lower.ends_with(".gz")
}

/// Strips leading directory prefixes and `.gz` suffix, matching reference `stripPathAndGz`.
pub fn clean_log_name(name: &str) -> String {
    let norm = name.replace('\\', "/");
    let file_name = norm.rsplit('/').next().unwrap_or(&norm);
    if let Some(stripped) = file_name.strip_suffix(".gz").or_else(|| file_name.strip_suffix(".GZ")) {
        stripped.to_string()
    } else {
        file_name.to_string()
    }
}

/// Decompresses raw gzip bytes.
pub fn decompress_gzip(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = GzDecoder::new(bytes);
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .map_err(|e| format!("Gzip decompression failed: {e}"))?;
    Ok(output)
}

/// Extracts a `.gz` file from disk.
pub fn extract_single_gz(path: &Path) -> Result<ExtractedArchive, String> {
    let start = Instant::now();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unnamed.gz".to_string());
    let clean_name = clean_log_name(&file_name);

    let mut file = File::open(path).map_err(|e| format!("Failed to open {}: {e}", path.display()))?;
    let mut decoder = GzDecoder::new(&mut file);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("Failed to decompress {}: {e}", path.display()))?;

    let kind = classify_log(&clean_name, &decompressed);
    let mut archive = ExtractedArchive {
        duration_ms: start.elapsed().as_millis() as u64,
        ..Default::default()
    };

    if kind == LogKind::Skip {
        archive.skipped.push(file_name);
        return Ok(archive);
    }

    let source = LoadedSource::Memory {
        name: clean_name,
        bytes: Arc::new(decompressed),
    };

    match kind {
        LogKind::Pm2 => archive.pm2_sources.push(source),
        LogKind::Mongo => archive.mongo_sources.push(source),
        LogKind::Unknown => archive.unknown_sources.push(source),
        LogKind::Skip => {}
    }

    Ok(archive)
}

/// Extracts a `.zip` archive from disk, handling nested `.gz` entries and classification.
pub fn extract_zip(path: &Path) -> Result<ExtractedArchive, String> {
    let start = Instant::now();
    let file = File::open(path).map_err(|e| format!("Failed to open {}: {e}", path.display()))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid or unsupported ZIP archive: {e}"))?;

    let mut pm2_sources = Vec::new();
    let mut mongo_sources = Vec::new();
    let mut unknown_sources = Vec::new();
    let mut skipped = Vec::new();

    for i in 0..zip.len() {
        let mut entry = match zip.by_index(i) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: skipping ZIP entry {i}: {e}");
                continue;
            }
        };

        let raw_name = entry.name().to_string();
        let clean_name = clean_log_name(&raw_name);

        // Skip directories and metadata
        if entry.is_dir()
            || entry.size() == 0
            || clean_name.starts_with('.')
            || clean_name.starts_with("__MACOSX")
            || clean_name.contains("error")
        {
            skipped.push(raw_name);
            continue;
        }

        let mut entry_bytes = Vec::with_capacity(entry.size() as usize);
        if let Err(e) = entry.read_to_end(&mut entry_bytes) {
            eprintln!("Warning: failed reading ZIP entry {raw_name}: {e}");
            skipped.push(raw_name);
            continue;
        }

        // Support nested gzip files within the zip archive
        let final_bytes = if entry_bytes.len() >= 2
            && entry_bytes[0] == 0x1f
            && entry_bytes[1] == 0x8b
        {
            match decompress_gzip(&entry_bytes) {
                Ok(decompressed) => decompressed,
                Err(_) => entry_bytes,
            }
        } else {
            entry_bytes
        };

        let kind = classify_log(&clean_name, &final_bytes);
        if kind == LogKind::Skip {
            skipped.push(raw_name);
            continue;
        }

        let source = LoadedSource::Memory {
            name: clean_name,
            bytes: Arc::new(final_bytes),
        };

        match kind {
            LogKind::Pm2 => pm2_sources.push(source),
            LogKind::Mongo => mongo_sources.push(source),
            LogKind::Unknown => unknown_sources.push(source),
            LogKind::Skip => {}
        }
    }

    Ok(ExtractedArchive {
        pm2_sources,
        mongo_sources,
        unknown_sources,
        skipped,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Automatically extracts either a `.zip` or a `.gz` file.
pub fn extract_archive(path: &Path) -> Result<ExtractedArchive, String> {
    let lower = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if lower.ends_with(".zip") {
        extract_zip(path)
    } else if lower.ends_with(".gz") {
        extract_single_gz(path)
    } else {
        Err(format!("Unsupported archive format: {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    #[test]
    fn test_clean_log_name() {
        assert_eq!(clean_log_name("api-out.log.gz"), "api-out.log");
        assert_eq!(clean_log_name("nested/path/mongod.log.1.gz"), "mongod.log.1");
        assert_eq!(clean_log_name("pm2.log"), "pm2.log");
    }

    #[test]
    fn test_zip_extraction_and_classification() {
        let temp_dir = std::env::temp_dir();
        let zip_path = temp_dir.join("test_archive.zip");

        // Create in-memory zip
        {
            let file = File::create(&zip_path).expect("create test zip");
            let mut zip = zip::ZipWriter::new(file);
            let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            zip.start_file("api-out.log", options).unwrap();
            zip.write_all(b"2026-07-24T00:00:01: GET /api/test 200 10.0 ms - 12\n").unwrap();

            zip.start_file("mongod.log", options).unwrap();
            zip.write_all(br#"{"t":{"$date":"2026-07-24T12:00:00Z"},"s":"I","msg":"Slow query"}"#).unwrap();

            zip.start_file("error.log", options).unwrap();
            zip.write_all(b"Error: some crash\n").unwrap();

            zip.finish().unwrap();
        }

        let result = extract_archive(&zip_path).expect("extract test zip");
        assert_eq!(result.pm2_sources.len(), 1);
        assert_eq!(result.mongo_sources.len(), 1);
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.pm2_sources[0].name(), "api-out.log");
        assert_eq!(result.mongo_sources[0].name(), "mongod.log");

        let _ = std::fs::remove_file(zip_path);
    }
}
