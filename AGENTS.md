# AGENTS.md

Operating instructions for coding agents working on PM2 Log Analyzer Native.

## Project Context
- Language & Edition: Rust 2021
- GUI Framework: egui 0.29 + eframe
- Engine: Memory mapping (`memmap2`), SIMD scanning (`memchr`), Multi-threaded parsing (`rayon`), fast hashing (`rapidhash`)
- Platform Target: Pure Native Windows `.exe`

## Building and Testing
- Check compilation: `cargo check`
- Run locally: `cargo run`
- Build release executable: `cargo build --release` (Produces `target/release/pm2-log-analyzer.exe`)
