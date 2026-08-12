use dioxus::prelude::*;
use crate::core::aggregator::{parse_log_buffer, Engine};
use crate::core::mmap::MmapReader;
use crate::core::models::{EndpointStats, FilterOptions, LogAnalysisSummary};
use crate::ui::*;
use std::path::PathBuf;
use std::sync::Arc;

const STYLE_CSS: &str = include_str!("../assets/style.css");

#[component]
pub fn App() -> Element {
    let theme = use_signal(|| "light".to_string());
    let mut source_kind = use_signal(|| "none".to_string());
    let mut file_name = use_signal(|| None::<String>);
    let mut file_size = use_signal(|| 0u64);
    let mut has_data = use_signal(|| false);
    let mut is_parsing = use_signal(|| false);
    let mut progress_percent = use_signal(|| 0u32);
    let mut status_msg = use_signal(|| "Ready".to_string());

    let mut engine = use_signal(|| None::<Arc<Engine>>);
    let mut summary = use_signal(|| LogAnalysisSummary::default());
    let mut sorted_endpoints = use_signal(|| Vec::<EndpointStats>::new());

    let filters = use_signal(|| FilterOptions::default());
    let sort_key = use_signal(|| "p95Ms".to_string());
    let top_n = use_signal(|| 50usize);
    let selected_methods = use_signal(|| Vec::<String>::new());

    let mut cron_jobs = use_signal(|| Vec::<crate::core::models::CronJobStats>::new());
    let mut unmatched_count = use_signal(|| 0u64);
    let mut unmatched_samples = use_signal(|| Vec::<String>::new());

    let mut toast_msg = use_signal(|| None::<String>);

    let mut show_toast = move |msg: String| {
        toast_msg.set(Some(msg));
        spawn(async move {
            tokio_time_sleep().await;
            toast_msg.set(None);
        });
    };

    let mut recompute = move || {
        if let Some(ref eng) = *engine.read() {
            let f = filters();
            let sum = eng.aggregate(file_size(), 0, &f);
            let mut sorted = sum.endpoints.clone();
            
            let sel_m = selected_methods();
            if !sel_m.is_empty() {
                sorted.retain(|e| sel_m.contains(&e.method.as_str().to_string()));
            }

            let sk = sort_key();
            sorted.sort_by(|a, b| {
                let cmp = match sk.as_str() {
                    "count" => b.total_calls.cmp(&a.total_calls),
                    "avgMs" => b.avg_duration_ms().partial_cmp(&a.avg_duration_ms()).unwrap_or(std::cmp::Ordering::Equal),
                    "maxMs" => b.max_duration_ms.partial_cmp(&a.max_duration_ms).unwrap_or(std::cmp::Ordering::Equal),
                    "p99Ms" => b.p99_ms.partial_cmp(&a.p99_ms).unwrap_or(std::cmp::Ordering::Equal),
                    "errorCount" => b.failed_calls.cmp(&a.failed_calls),
                    _ => b.p95_ms.partial_cmp(&a.p95_ms).unwrap_or(std::cmp::Ordering::Equal),
                };
                cmp
            });

            sorted.truncate(top_n());
            cron_jobs.set(sum.cron_jobs.clone());
            unmatched_count.set(sum.unmatched_lines as u64);
            unmatched_samples.set(sum.unmatched_samples.clone());
            sorted_endpoints.set(sorted);
            summary.set(sum);
        }
    };

    let mut load_file = move |path: PathBuf| {
        is_parsing.set(true);
        progress_percent.set(20);
        status_msg.set(format!("Opening {}", path.display()));
        source_kind.set("file".to_string());
        file_name.set(Some(path.file_name().unwrap_or_default().to_string_lossy().to_string()));

        let f_opts = filters();

        spawn(async move {
            let res = tokio::task::spawn_blocking(move || {
                let start = std::time::Instant::now();
                if let Ok(reader) = MmapReader::open(&path) {
                    let f_size = reader.len() as u64;
                    let eng = parse_log_buffer(reader.as_slice());
                    let elapsed_ms = start.elapsed().as_millis() as u64;
                    let sum = eng.aggregate(f_size, elapsed_ms, &f_opts);

                    let arc_eng = Arc::new(eng);
                    let mut sorted = sum.endpoints.clone();
                    sorted.sort_by(|a, b| b.p95_ms.partial_cmp(&a.p95_ms).unwrap_or(std::cmp::Ordering::Equal));
                    sorted.truncate(50);

                    Ok((f_size, arc_eng, sum, sorted, elapsed_ms))
                } else {
                    Err("Failed to open file".to_string())
                }
            }).await;

            match res {
                Ok(Ok((f_size, arc_eng, sum, sorted, elapsed_ms))) => {
                    file_size.set(f_size);
                    engine.set(Some(arc_eng));
                    cron_jobs.set(sum.cron_jobs.clone());
                    unmatched_count.set(sum.unmatched_lines as u64);
                    unmatched_samples.set(sum.unmatched_samples.clone());
                    summary.set(sum);
                    sorted_endpoints.set(sorted);
                    has_data.set(true);
                    is_parsing.set(false);
                    progress_percent.set(100);
                    status_msg.set(format!("Parsed in {} ms", elapsed_ms));
                }
                _ => {
                    is_parsing.set(false);
                    status_msg.set("Failed to open file".to_string());
                }
            }
        });
    };

    let mut analyze_paste = move |text: String| {
        is_parsing.set(true);
        progress_percent.set(30);
        status_msg.set("Analyzing pasted text...".to_string());
        source_kind.set("paste".to_string());
        file_name.set(None);
        let f_size_val = text.len() as u64;
        file_size.set(f_size_val);

        let f_opts = filters();

        spawn(async move {
            let res = tokio::task::spawn_blocking(move || {
                let start = std::time::Instant::now();
                let eng = parse_log_buffer(text.as_bytes());
                let elapsed_ms = start.elapsed().as_millis() as u64;
                let sum = eng.aggregate(text.len() as u64, elapsed_ms, &f_opts);

                let arc_eng = Arc::new(eng);
                let mut sorted = sum.endpoints.clone();
                sorted.sort_by(|a, b| b.p95_ms.partial_cmp(&a.p95_ms).unwrap_or(std::cmp::Ordering::Equal));
                sorted.truncate(50);

                (arc_eng, sum, sorted, elapsed_ms)
            }).await;

            if let Ok((arc_eng, sum, sorted, elapsed_ms)) = res {
                engine.set(Some(arc_eng));
                cron_jobs.set(sum.cron_jobs.clone());
                unmatched_count.set(sum.unmatched_lines as u64);
                unmatched_samples.set(sum.unmatched_samples.clone());
                summary.set(sum);
                sorted_endpoints.set(sorted);
                has_data.set(true);
                is_parsing.set(false);
                progress_percent.set(100);
                status_msg.set(format!("Parsed paste in {} ms", elapsed_ms));
            } else {
                is_parsing.set(false);
                status_msg.set("Failed to analyze paste".to_string());
            }
        });
    };

    let clear_all = move |_| {
        source_kind.set("none".to_string());
        file_name.set(None);
        file_size.set(0);
        has_data.set(false);
        engine.set(None);
        summary.set(LogAnalysisSummary::default());
        sorted_endpoints.set(Vec::new());
        cron_jobs.set(Vec::new());
        unmatched_count.set(0);
        unmatched_samples.set(Vec::new());
        is_parsing.set(false);
        show_toast("Cleared loaded log data".to_string());
    };

    let is_dark = theme() == "dark";

    rsx! {
        style { "{STYLE_CSS}" }
        div { class: if is_dark { "dark min-h-full bg-[#0b0f19] text-slate-100 font-sans" } else { "min-h-full bg-[#f7f8fa] text-slate-900 font-sans" },
            AppHeader {
                theme: theme,
                source_kind: source_kind,
                file_name: file_name,
                file_size: file_size,
                has_data: has_data,
                is_parsing: is_parsing,
                summary: summary,
                on_toast: move |msg| show_toast(msg),
                on_clear: clear_all,
            }
            main { class: "mx-auto max-w-7xl space-y-4 px-4 py-4",
                IngestPanel {
                    has_data: has_data,
                    is_parsing: is_parsing,
                    progress_percent: progress_percent,
                    on_file_select: move |path| load_file(path),
                    on_paste_analyze: move |text| analyze_paste(text),
                    on_toast: move |msg| show_toast(msg),
                }

                KpiRow {
                    has_data: has_data,
                    summary: summary,
                }

                FilterBar {
                    has_data: has_data,
                    filters: filters,
                    sort_key: sort_key,
                    top_n: top_n,
                    selected_methods: selected_methods,
                    on_filter_change: move |_| recompute(),
                }

                div { class: "grid gap-4 lg:grid-cols-5",
                    div { class: "lg:col-span-3",
                        ApiTable {
                            endpoints: sorted_endpoints,
                            on_toast: move |msg| show_toast(msg),
                        }
                    }
                    div { class: "lg:col-span-2",
                        LatencyChart {
                            has_data: has_data,
                            summary: summary,
                            endpoints: sorted_endpoints,
                            theme: theme,
                        }
                    }
                }

                CronTable {
                    jobs: cron_jobs,
                    on_toast: move |msg| show_toast(msg),
                }

                SkippedDisclosure {
                    unmatched_count: unmatched_count,
                    samples: unmatched_samples,
                    has_data: has_data,
                }

                footer { class: "pb-6 pt-2 text-center text-[11px] text-slate-400 dark:text-slate-500",
                    "Parses in your native Windows engine - logs never leave this machine"
                }
            }
            Toast {
                toast_msg: toast_msg,
            }
        }
    }
}

async fn tokio_time_sleep() {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(3000).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
}
