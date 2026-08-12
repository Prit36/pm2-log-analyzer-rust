use dioxus::prelude::*;
use crate::core::models::LogAnalysisSummary;

fn format_num(val: u64) -> String {
    let s = val.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

fn format_ms(val: f32) -> String {
    if val >= 1000.0 {
        format!("{:.2} s", val / 1000.0)
    } else {
        format!("{:.1} ms", val)
    }
}

#[component]
pub fn KpiRow(
    has_data: Signal<bool>,
    summary: Signal<LogAnalysisSummary>,
) -> Element {
    if !has_data() || summary().total_lines_parsed == 0 {
        return rsx! {};
    }

    let sum = summary();
    let requests = format_num(sum.matched_http_requests as u64);
    let avg = format_ms(sum.avg_latency_ms());
    let p95 = format_ms(sum.p95_latency_ms());
    let errors_cnt = sum.error_responses();
    let errors = format_num(errors_cnt);
    let slow = format_num(sum.slow_requests_3s());
    let cron_jobs = sum.cron_jobs.len();
    let has_cron = cron_jobs > 0;

    let card_class = "bg-white px-3 py-2.5 dark:bg-slate-900";

    rsx! {
        section { class: "grid grid-cols-2 gap-px overflow-hidden rounded border border-slate-200 bg-slate-200 sm:grid-cols-3 dark:border-slate-800 dark:bg-slate-800 lg:grid-cols-6",
            div { class: "{card_class}",
                div { class: "text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                    "Requests"
                }
                div { class: "mt-1 font-mono-data text-lg font-semibold tabular-nums text-slate-900 dark:text-slate-100",
                    "{requests}"
                }
            }
            div { class: "{card_class}",
                div { class: "text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                    "Avg"
                }
                div { class: "mt-1 font-mono-data text-lg font-semibold tabular-nums text-slate-900 dark:text-slate-100",
                    "{avg}"
                }
            }
            div { class: "{card_class}",
                div { class: "text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                    "p95"
                }
                div { class: "mt-1 font-mono-data text-lg font-semibold tabular-nums text-blue-600 dark:text-blue-400",
                    "{p95}"
                }
            }
            div { class: "{card_class}",
                div { class: "text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                    "Errors"
                }
                div {
                    class: if errors_cnt > 0 { "mt-1 font-mono-data text-lg font-semibold tabular-nums text-rose-600 dark:text-rose-400" } else { "mt-1 font-mono-data text-lg font-semibold tabular-nums text-slate-900 dark:text-slate-100" },
                    "{errors}"
                }
            }
            div { class: "{card_class}",
                div { class: "text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                    "Slow ≥3s"
                }
                div { class: "mt-1 font-mono-data text-lg font-semibold tabular-nums text-slate-900 dark:text-slate-100",
                    "{slow}"
                }
            }
            if has_cron {
                div { class: "{card_class}",
                    div { class: "text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Cron jobs"
                    }
                    div { class: "mt-1 font-mono-data text-lg font-semibold tabular-nums text-slate-900 dark:text-slate-100",
                        "{cron_jobs}"
                    }
                }
            }
        }
    }
}
