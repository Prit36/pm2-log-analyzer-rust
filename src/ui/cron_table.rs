//! `CronTable` — port of `src/components/CronTable.tsx`.

use dioxus::prelude::*;

use crate::core::models::CronAggregated;
use crate::store::{
    reaggregate, set_filters, show_toast, use_analysis_store, CronSortKey, SortDirection,
};
use crate::ui::icons::Icon;
use crate::utils::cn::cn;
use crate::utils::format::{format_ms, format_num};
use crate::utils::table_ops::{build_cron_tsv, copy_to_clipboard};

const FIELD_CLASS: &str = "rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-800 focus:border-blue-500 dark:border-slate-700 dark:bg-slate-950 dark:text-slate-100 dark:focus:border-blue-400";

fn cron_sort_key_str(key: CronSortKey) -> &'static str {
    match key {
        CronSortKey::P95Ms => "p95Ms",
        CronSortKey::P99Ms => "p99Ms",
        CronSortKey::AvgMs => "avgMs",
        CronSortKey::MaxMs => "maxMs",
        CronSortKey::Runs => "runs",
        CronSortKey::Starts => "starts",
        CronSortKey::Fails => "fails",
        CronSortKey::LastDurationMs => "lastDurationMs",
        CronSortKey::Name => "name",
    }
}

fn parse_cron_sort_key(value: &str) -> Option<CronSortKey> {
    match value {
        "p95Ms" => Some(CronSortKey::P95Ms),
        "p99Ms" => Some(CronSortKey::P99Ms),
        "avgMs" => Some(CronSortKey::AvgMs),
        "maxMs" => Some(CronSortKey::MaxMs),
        "runs" => Some(CronSortKey::Runs),
        "starts" => Some(CronSortKey::Starts),
        "fails" => Some(CronSortKey::Fails),
        "lastDurationMs" => Some(CronSortKey::LastDurationMs),
        "name" => Some(CronSortKey::Name),
        _ => None,
    }
}

#[component]
pub fn CronTable(rows: Vec<CronAggregated>) -> Element {
    let store = use_analysis_store();
    let filters = store.filters();
    let height = 360.0_f64.min(120.0_f64.max(rows.len() as f64 * 32.0 + 36.0));
    let rows_for_copy = rows.clone();

    rsx! {
        section { class: "overflow-hidden rounded border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900",
            div { class: "flex flex-wrap items-center justify-between gap-2 border-b border-slate-200 px-3 py-2 dark:border-slate-800",
                h2 { class: "text-xs font-semibold uppercase tracking-wide text-slate-600 dark:text-slate-300",
                    "Cron jobs"
                }
                div { class: "flex flex-wrap items-center gap-2",
                    input {
                        r#type: "search",
                        value: "{filters.cron_query}",
                        oninput: move |e: Event<FormData>| {
                            let mut f = store.filters();
                            f.cron_query = e.value();
                            set_filters(store, f);
                            reaggregate(store);
                        },
                        placeholder: "Filter jobs…",
                        class: cn(&[FIELD_CLASS, "w-40"]),
                    }
                    input {
                        r#type: "number",
                        min: "0",
                        value: "{fmt_ms_input(filters.cron_min_ms)}",
                        oninput: move |e: Event<FormData>| {
                            let mut f = store.filters();
                            f.cron_min_ms = e.value().parse::<f64>().unwrap_or(0.0).max(0.0);
                            set_filters(store, f);
                            reaggregate(store);
                        },
                        title: "Min duration ms",
                        class: cn(&[FIELD_CLASS, "w-20"]),
                        placeholder: "Min ms",
                    }
                    label { class: "flex items-center gap-1.5 text-[11px] text-slate-600 dark:text-slate-400",
                        input {
                            r#type: "checkbox",
                            checked: filters.cron_show_failed_only,
                            onchange: move |e: Event<FormData>| {
                                let mut f = store.filters();
                                f.cron_show_failed_only = e.checked();
                                set_filters(store, f);
                                reaggregate(store);
                            },
                        }
                        "Failures only"
                    }
                    select {
                        value: "{cron_sort_key_str(filters.cron_sort_key)}",
                        onchange: move |e: Event<FormData>| {
                            if let Some(key) = parse_cron_sort_key(&e.value()) {
                                let mut f = store.filters();
                                f.cron_sort_key = key;
                                f.cron_sort_dir = if key == CronSortKey::Name {
                                    SortDirection::Asc
                                } else {
                                    SortDirection::Desc
                                };
                                set_filters(store, f);
                            }
                        },
                        class: "{FIELD_CLASS}",
                        option { value: "p95Ms", "p95" }
                        option { value: "p99Ms", "p99" }
                        option { value: "avgMs", "avg" }
                        option { value: "maxMs", "max" }
                        option { value: "runs", "runs" }
                        option { value: "starts", "starts" }
                        option { value: "fails", "fails" }
                        option { value: "lastDurationMs", "last" }
                        option { value: "name", "job" }
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| {
                            if rows_for_copy.is_empty() {
                                return;
                            }
                            if copy_to_clipboard(&build_cron_tsv(&rows_for_copy)) {
                                show_toast(store, "Cron table copied — paste into Excel");
                            }
                        },
                        disabled: rows.is_empty(),
                        class: "inline-flex items-center gap-1 text-[11px] font-medium text-slate-500 hover:text-slate-800 disabled:opacity-40 dark:text-slate-400 dark:hover:text-slate-200 cursor-pointer border-0 bg-transparent",
                        Icon { name: "copy", class: "size-3" }
                        "Copy TSV"
                    }
                }
            }
            if rows.is_empty() {
                div { class: "px-3 py-8 text-center text-sm text-slate-400 dark:text-slate-500",
                    "No cron jobs match filters."
                }
            } else {
                div { style: "height: {height}px;",
                    div { class: "grid grid-cols-[minmax(0,1.2fr)_56px_56px_56px_64px_64px_64px_64px_64px] items-center gap-1 border-b border-slate-100 bg-slate-50 px-3 py-2 text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:border-slate-800 dark:bg-slate-950 dark:text-slate-400",
                        CronSortHeader {
                            label: "Job",
                            col_key: CronSortKey::Name,
                            current_key: filters.cron_sort_key,
                            current_dir: filters.cron_sort_dir,
                            align_left: true,
                        }
                        CronSortHeader {
                            label: "Runs",
                            col_key: CronSortKey::Runs,
                            current_key: filters.cron_sort_key,
                            current_dir: filters.cron_sort_dir,
                            align_left: false,
                        }
                        CronSortHeader {
                            label: "Starts",
                            col_key: CronSortKey::Starts,
                            current_key: filters.cron_sort_key,
                            current_dir: filters.cron_sort_dir,
                            align_left: false,
                        }
                        CronSortHeader {
                            label: "Fails",
                            col_key: CronSortKey::Fails,
                            current_key: filters.cron_sort_key,
                            current_dir: filters.cron_sort_dir,
                            align_left: false,
                        }
                        CronSortHeader {
                            label: "Avg",
                            col_key: CronSortKey::AvgMs,
                            current_key: filters.cron_sort_key,
                            current_dir: filters.cron_sort_dir,
                            align_left: false,
                        }
                        CronSortHeader {
                            label: "p95",
                            col_key: CronSortKey::P95Ms,
                            current_key: filters.cron_sort_key,
                            current_dir: filters.cron_sort_dir,
                            align_left: false,
                        }
                        CronSortHeader {
                            label: "p99",
                            col_key: CronSortKey::P99Ms,
                            current_key: filters.cron_sort_key,
                            current_dir: filters.cron_sort_dir,
                            align_left: false,
                        }
                        CronSortHeader {
                            label: "Max",
                            col_key: CronSortKey::MaxMs,
                            current_key: filters.cron_sort_key,
                            current_dir: filters.cron_sort_dir,
                            align_left: false,
                        }
                        CronSortHeader {
                            label: "Last",
                            col_key: CronSortKey::LastDurationMs,
                            current_key: filters.cron_sort_key,
                            current_dir: filters.cron_sort_dir,
                            align_left: false,
                        }
                    }
                    div { class: "overflow-auto", style: "height: {height - 36.0}px;",
                        div { style: "height: {rows.len() * 32}px; width: 100%;",
                            for (index, row) in rows.iter().enumerate() {
                                {
                                    let is_even = index % 2 == 0;
                                    let row_bg = if is_even { "bg-white dark:bg-slate-900" } else { "bg-slate-50/50 dark:bg-slate-950/40" };
                                    let fails_class = if row.fails > 0 {
                                        "text-right tabular-nums font-semibold text-rose-600 dark:text-rose-400"
                                    } else {
                                        "text-right tabular-nums text-slate-400 dark:text-slate-600"
                                    };
                                    rsx! {
                                        div {
                                            key: "{row.name}",
                                            class: cn(&[
                                                "grid grid-cols-[minmax(0,1.2fr)_56px_56px_56px_64px_64px_64px_64px_64px] items-center gap-1 border-b border-slate-100 px-3 text-xs dark:border-slate-800",
                                                row_bg,
                                            ]),
                                            style: "height: 32px;",
                                            div { class: "truncate font-mono-data text-[11px] text-slate-800 dark:text-slate-200",
                                                "{row.name}"
                                            }
                                            div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300",
                                                "{format_num(row.runs)}"
                                            }
                                            div { class: "text-right tabular-nums text-slate-400 dark:text-slate-500",
                                                "{format_num(row.starts)}"
                                            }
                                            div { class: "{fails_class}", "{format_num(row.fails)}" }
                                            div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300",
                                                "{format_ms(row.avg_ms)}"
                                            }
                                            div { class: "text-right tabular-nums font-semibold text-blue-600 dark:text-blue-400",
                                                "{format_ms(row.p95_ms)}"
                                            }
                                            div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300",
                                                "{format_ms(row.p99_ms)}"
                                            }
                                            div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300",
                                                "{format_ms(row.max_ms)}"
                                            }
                                            div { class: "text-right tabular-nums text-slate-400 dark:text-slate-500",
                                                {match row.last_duration_ms {
                                                    Some(ms) => format_ms(ms),
                                                    None => "-".to_string(),
                                                }}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn fmt_ms_input(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

#[component]
fn CronSortHeader(
    label: String,
    col_key: CronSortKey,
    current_key: CronSortKey,
    current_dir: SortDirection,
    align_left: bool,
) -> Element {
    let store = use_analysis_store();
    let is_active = current_key == col_key;
    let title = format!(
        "Sort by {label} ({})",
        if is_active && current_dir == SortDirection::Desc {
            "descending"
        } else {
            "ascending"
        }
    );
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| {
                let mut f = store.filters();
                if f.cron_sort_key == col_key {
                    f.cron_sort_dir = if f.cron_sort_dir == SortDirection::Asc {
                        SortDirection::Desc
                    } else {
                        SortDirection::Asc
                    };
                } else {
                    f.cron_sort_key = col_key;
                    f.cron_sort_dir = if col_key == CronSortKey::Name {
                        SortDirection::Asc
                    } else {
                        SortDirection::Desc
                    };
                }
                set_filters(store, f);
            },
            class: cn(&[
                "group flex w-full items-center gap-1 cursor-pointer select-none transition-colors border-0 bg-transparent",
                if align_left { "justify-start text-left" } else { "justify-end text-right" },
                if is_active {
                    "font-bold text-blue-600 dark:text-blue-400"
                } else {
                    "text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200"
                },
            ]),
            title: "{title}",
            span { "{label}" }
            if is_active {
                if current_dir == SortDirection::Asc {
                    Icon { name: "arrow-up", class: "size-3 shrink-0" }
                } else {
                    Icon { name: "arrow-down", class: "size-3 shrink-0" }
                }
            } else {
                Icon { name: "arrow-up-down", class: "size-2.5 shrink-0 opacity-0 group-hover:opacity-60 transition-opacity" }
            }
        }
    }
}
