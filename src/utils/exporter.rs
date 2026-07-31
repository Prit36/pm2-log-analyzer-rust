use crate::core::models::LogAnalysisSummary;
use std::fs::File;
use std::io::{BufWriter, Write};

pub fn export_to_csv(summary: &LogAnalysisSummary, path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "Method,Endpoint,Calls,ErrorCalls,ErrorRate(%),AvgDuration(ms),p50(ms),p95(ms),p99(ms)")?;

    for ep in &summary.endpoints {
        writeln!(
            writer,
            "\"{}\",\"{}\",{},{},{:.2},{:.2},{:.2},{:.2},{:.2}",
            ep.method.as_str(),
            ep.path,
            ep.total_calls,
            ep.error_calls,
            ep.error_rate(),
            ep.avg_duration_ms(),
            ep.p50_ms,
            ep.p95_ms,
            ep.p99_ms
        )?;
    }
    writer.flush()?;
    Ok(())
}

pub fn export_to_json(summary: &LogAnalysisSummary, path: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, summary)?;
    writer.flush()?;
    Ok(())
}
