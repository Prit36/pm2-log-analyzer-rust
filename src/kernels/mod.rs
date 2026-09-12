//! Vendored Wasm kernels (verbatim from `../pm2-log-analyzer/wasm/*`), minus the
//! wasm-bindgen wrappers. These are the parity source of truth for parsing and
//! aggregation; do not "improve" them without changing the reference first.

pub mod mongo;
pub mod pm2;
