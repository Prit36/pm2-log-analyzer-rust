#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod app;
pub mod core;
pub mod kernels;
pub mod store;
pub mod ui;
pub mod utils;
