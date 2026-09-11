//! `KpiRow` — port of `src/components/KpiRow.tsx`.

use dioxus::prelude::*;

use crate::store::use_analysis_store;
use crate::utils::cn::cn;
use crate::utils::format::{format_ms, format_num};

struct KpiItem {
    label: &'static str,
    value: String,
    accent: bool,
    danger: bool,
}

#[component]
pub fn KpiRow() -> Element {
    let store = use_analysis_store();
    let Some(summary) = store.summary() else {
        return rsx! {};
    };

    let mut items = vec![
        KpiItem {
            label: "Requests",
            value: format_num(summary.matched),
            accent: false,
            danger: false,
        },
        KpiItem {
            label: "Avg",
            value: format_ms(summary.avg),
            accent: false,
            danger: false,
        },
        KpiItem {
            label: "p95",
            value: format_ms(summary.p95_ms),
            accent: true,
            danger: false,
        },
        KpiItem {
            label: "Errors",
            value: format_num(summary.errors),
            accent: false,
            danger: summary.errors > 0,
        },
        KpiItem {
            label: "Slow ≥3s",
            value: format_num(summary.slow),
            accent: false,
            danger: false,
        },
    ];

    if store.has_cron_events() {
        items.push(KpiItem {
            label: "Cron jobs",
            value: format_num(store.result().map(|r| r.cron_summary.jobs).unwrap_or(0)),
            accent: false,
            danger: false,
        });
    }

    rsx! {
        section {
            "data-testid": "kpi-row",
            class: "grid grid-cols-2 gap-px overflow-hidden rounded border border-slate-200 bg-slate-200 sm:grid-cols-3 dark:border-slate-800 dark:bg-slate-800 lg:grid-flow-col lg:auto-cols-fr",
            for item in items {
                div { key: "{item.label}", class: "bg-white px-3 py-3 dark:bg-slate-900",
                    div { class: "text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "{item.label}"
                    }
                    div {
                        class: cn(&[
                            "mt-1 font-mono-data text-lg font-semibold tabular-nums",
                            if item.danger {
                                "text-rose-600 dark:text-rose-400"
                            } else if item.accent {
                                "text-blue-600 dark:text-blue-400"
                            } else {
                                "text-slate-900 dark:text-slate-100"
                            },
                        ]),
                        "{item.value}"
                    }
                }
            }
        }
    }
}
