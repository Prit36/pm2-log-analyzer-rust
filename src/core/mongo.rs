//! Native MongoDB pipeline — mirrors `src/workers/mongoParserWorker.ts`.
//!
//! The vendored kernel does the parsing and aggregation; this module only drives
//! the ingest window (16 MiB chunks, same as the worker) and decodes the kernel's
//! JSON result into the typed model.

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use memmap2::Mmap;

use crate::core::mongo_models::{MongoAggregationResult, MongoFilters};
use crate::core::pm2::{JobControl, LoadedSource, ParseError};
use crate::kernels::mongo::reagg::FilterParams;
use crate::kernels::mongo::store::Engine;

const FEED_CHUNK: usize = 16 * 1024 * 1024;

/// Parsed MongoDB logs: the columnar kernel plus its line counters.
pub struct MongoKernel {
    engine: Engine,
}

impl MongoKernel {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
        }
    }

    /// Feeds every source through the kernel in file order.
    pub fn parse_sources(
        &mut self,
        sources: &[LoadedSource],
        control: &JobControl,
    ) -> Result<(), ParseError> {
        for source in sources {
            if control.cancelled.load(Ordering::Relaxed) {
                return Err(ParseError::Cancelled);
            }
            match source {
                LoadedSource::Path(path) => self.feed_path(path, control)?,
                LoadedSource::Memory { bytes, .. } => self.feed_bytes(bytes, control),
            }
        }
        self.engine.end_shard();
        Ok(())
    }

    fn feed_path(&mut self, path: &Path, control: &JobControl) -> Result<(), ParseError> {
        let file = std::fs::File::open(path).map_err(|error| ParseError::Io(error.to_string()))?;
        let size = file
            .metadata()
            .map_err(|error| ParseError::Io(error.to_string()))?
            .len() as usize;
        if size == 0 {
            return Ok(());
        }
        // SAFETY: the mapping outlives every read below and the file is not mutated.
        let mmap = unsafe { Mmap::map(&file) }.map_err(|error| ParseError::Io(error.to_string()))?;
        self.feed_bytes(&mmap, control);
        Ok(())
    }

    fn feed_bytes(&mut self, data: &[u8], control: &JobControl) {
        let mut offset = 0usize;
        while offset < data.len() {
            if control.cancelled.load(Ordering::Relaxed) {
                return;
            }
            let take = (data.len() - offset).min(FEED_CHUNK);
            let len = self.engine.ingest_ptr(take as u32);
            debug_assert!(len != 0);
            self.engine.ingest[..take].copy_from_slice(&data[offset..offset + take]);
            self.engine.feed(take as u32, offset as u64);
            offset += take;
            control
                .processed
                .fetch_add(take as u64, Ordering::Relaxed);
        }
    }

    /// Fast filtered aggregation (microseconds — safe to call on every keystroke).
    pub fn reaggregate(&self, filters: &MongoFilters) -> MongoAggregationResult {
        let params = FilterParams {
            op: &filters.operation,
            plan_filter: filters.plan_filter.code(),
            min_duration_ms: filters.min_duration_ms,
            collection: &filters.collection,
            search_query: &filters.search_query,
            high_scan_ratio_only: filters.high_scan_ratio_only,
            user: &filters.user_filter,
        };
        let json = crate::kernels::mongo::reagg::reaggregate(&self.engine, params);
        serde_json::from_str(&json).unwrap_or_default()
    }

    pub fn slow_query_count(&self) -> u32 {
        self.engine.slow_query_count()
    }

    pub fn total_lines(&self) -> u64 {
        self.engine.total_lines as u64
    }
}

impl Default for MongoKernel {
    fn default() -> Self {
        Self::new()
    }
}

/// Reads every source and returns the parsed kernel (worker-thread entry point).
pub fn parse_paths(sources: &[LoadedSource], control: &JobControl) -> Result<MongoKernel, ParseError> {
    let mut kernel = MongoKernel::new();
    kernel.parse_sources(sources, control)?;
    Ok(kernel)
}

/// Paths of the sample logs shipped for manual parity checks.
pub fn sample_paths(root: &Path) -> Vec<PathBuf> {
    let dir = root.join("mongodb_logs_sample");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "log"))
                .collect()
        })
        .unwrap_or_default();
    paths.sort();
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mongo_models::MongoFilters;

    fn reference_sample() -> Option<PathBuf> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../pm2-log-analyzer/mongodb_logs_sample/eSanad-mongod.log");
        path.exists().then_some(path)
    }

    #[test]
    fn parses_the_reference_sample_and_reaggregates() {
        let Some(path) = reference_sample() else {
            return;
        };
        let size = std::fs::metadata(&path).expect("metadata").len();
        let control = JobControl::new(size);
        let kernel = parse_paths(&[LoadedSource::Path(path)], &control).expect("parse");
        assert!(kernel.total_lines() > 100, "sample must contain log lines");

        let result = kernel.reaggregate(&MongoFilters::default());
        assert!(
            result.summary.slow_query_count > 0,
            "sample must contain slow queries"
        );
        assert!(!result.patterns.is_empty(), "patterns must be derived");
        assert!(
            result.patterns.iter().all(|p| p.example_query.command.is_object()),
            "example queries keep their command document"
        );
        assert!(
            result.slow_queries.iter().all(|q| !q.timestamp.is_empty()),
            "every slow query carries a timestamp"
        );
        assert!(result.connections.accepted > 0, "connection stats parsed");

        // Filters flow through to the kernel: a duration floor must cut the set.
        let filtered = kernel.reaggregate(&MongoFilters {
            min_duration_ms: 5_000,
            ..MongoFilters::default()
        });
        assert!(filtered.slow_queries.len() < result.slow_queries.len());
    }
}
