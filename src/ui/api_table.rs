//! `ApiTable` — port of `src/components/ApiTable.tsx`.

use dioxus::prelude::*;

use crate::core::models::AggregatedEndpoint;
use crate::store::{show_toast, use_analysis_store, ApiSortKey, SortDirection};
use crate::ui::icons::Icon;
use crate::utils::cn::cn;
use crate::utils::format::{format_ms, format_num};
use crate::utils::table_ops::{build_api_tsv, copy_to_clipboard};

fn method_class(method: &str) -> &'static str {
    match method {
        "GET" => {
            "bg-sky-50 text-sky-700 ring-sky-200 dark:border dark:border-sky-600/60 dark:bg-[#062238] dark:text-[#38bdf8]"
        }
        "POST" => {
            "bg-emerald-50 text-emerald-700 ring-emerald-200 dark:border dark:border-emerald-600/60 dark:bg-[#06261c] dark:text-[#34d399]"
        }
        "PUT" | "PATCH" => {
            "bg-amber-50 text-amber-700 ring-amber-200 dark:border dark:border-amber-600/60 dark:bg-[#381a06] dark:text-[#fbbf24]"
        }
        "DELETE" => {
            "bg-rose-50 text-rose-700 ring-rose-200 dark:border dark:border-rose-600/60 dark:bg-[#3d0818] dark:text-[#fb7185]"
        }
        _ => {
            "bg-slate-50 text-slate-600 ring-slate-200 dark:border dark:border-slate-700/60 dark:bg-slate-900 dark:text-slate-400"
        }
    }
}

#[component]
pub fn ApiTable(rows: Vec<AggregatedEndpoint>) -> Element {
    let store = use_analysis_store();
    let filters = store.filters();
    let height = 420.0_f64.min(120.0_f64.max(rows.len() as f64 * 32.0 + 36.0));
    let rows_for_copy = rows.clone();

    rsx! {
        section { class: "overflow-hidden rounded border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900",
            div { class: "flex items-center justify-between border-b border-slate-200 px-3 py-2 dark:border-slate-800",
                h2 { class: "text-xs font-semibold uppercase tracking-wide text-slate-600 dark:text-slate-300",
                    "Slow API endpoints"
                }
                button {
                    r#type: "button",
                    onclick: move |_| {
                        if rows_for_copy.is_empty() {
                            return;
                        }
                        if copy_to_clipboard(&build_api_tsv(&rows_for_copy)) {
                            show_toast(store, "API table copied — paste into Excel");
                        }
                    },
                    disabled: rows.is_empty(),
                    class: "inline-flex items-center gap-1 text-[11px] font-medium text-slate-500 hover:text-slate-800 disabled:opacity-40 dark:text-slate-400 dark:hover:text-slate-200 cursor-pointer border-0 bg-transparent",
                    Icon { name: "copy", class: "size-3" }
                    "Copy TSV"
                }
            }
            if rows.is_empty() {
                div { class: "px-3 py-10 text-center text-sm text-slate-400 dark:text-slate-500",
                    "No matching endpoints"
                }
            } else {
                div { class: "overflow-x-auto",
                    div { class: "min-w-[680px]",
                        div { class: "grid grid-cols-[minmax(0,1fr)_56px_58px_58px_58px_58px_56px] border-b border-slate-200 bg-slate-50 px-3 py-2 text-[11px] font-semibold text-slate-500 dark:border-slate-800 dark:bg-slate-950 dark:text-slate-400",
                            div { "Endpoint" }
                            ApiSortHeader {
                                label: "Count",
                                col_key: ApiSortKey::Count,
                                current_key: filters.sort_key,
                                current_dir: filters.sort_dir,
                            }
                            ApiSortHeader {
                                label: "Avg",
                                col_key: ApiSortKey::AvgMs,
                                current_key: filters.sort_key,
                                current_dir: filters.sort_dir,
                            }
                            ApiSortHeader {
                                label: "p95",
                                col_key: ApiSortKey::P95Ms,
                                current_key: filters.sort_key,
                                current_dir: filters.sort_dir,
                            }
                            ApiSortHeader {
                                label: "p99",
                                col_key: ApiSortKey::P99Ms,
                                current_key: filters.sort_key,
                                current_dir: filters.sort_dir,
                            }
                            ApiSortHeader {
                                label: "Max",
                                col_key: ApiSortKey::MaxMs,
                                current_key: filters.sort_key,
                                current_dir: filters.sort_dir,
                            }
                            ApiSortHeader {
                                label: "Errors",
                                col_key: ApiSortKey::ErrorCount,
                                current_key: filters.sort_key,
                                current_dir: filters.sort_dir,
                            }
                        }
                        div { class: "overflow-auto", style: "height: {height}px;",
                            div { style: "height: {rows.len() * 32}px; width: 100%;",
                                for (index, row) in rows.iter().enumerate() {
                                    {
                                        let path = row.path.clone();
                                        let method = row.method.as_str();
                                        let is_even = index % 2 == 0;
                                        let row_bg = if is_even { "bg-white dark:bg-slate-900" } else { "bg-slate-50/50 dark:bg-slate-950/40" };
                                        let err_class = if row.error_count > 0 {
                                            "text-right tabular-nums font-semibold text-rose-600 dark:text-rose-500"
                                        } else {
                                            "text-right tabular-nums text-slate-400 dark:text-slate-600"
                                        };
                                        rsx! {
                                            div {
                                                key: "{row.key}",
                                                class: cn(&[
                                                    "grid grid-cols-[minmax(0,1fr)_56px_58px_58px_58px_58px_56px] items-center border-b border-slate-100 px-3 text-xs dark:border-slate-800",
                                                    row_bg,
                                                ]),
                                                style: "height: 32px;",
                                                div { class: "flex min-w-0 items-center gap-2 pr-2",
                                                    span {
                                                        class: cn(&[
                                                            "inline-block shrink-0 rounded px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide ring-1 dark:ring-0",
                                                            method_class(method),
                                                        ]),
                                                        "{method}"
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        onclick: move |_| {
                                                            if copy_to_clipboard(&path) {
                                                                show_toast(store, "Path copied");
                                                            }
                                                        },
                                                        title: "Click to copy: {row.path}",
                                                        class: "group flex min-w-0 flex-1 items-center gap-1.5 text-left font-mono-data text-[11px] text-slate-800 hover:text-blue-600 dark:text-slate-200 dark:hover:text-blue-400 cursor-pointer border-0 bg-transparent",
                                                        span { class: "truncate", "{row.path}" }
                                                        Icon { name: "copy", class: "size-3 shrink-0 opacity-0 group-hover:opacity-100 text-slate-400 transition-opacity" }
                                                    }
                                                }
                                                div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300",
                                                    "{format_num(row.count)}"
                                                }
                                                div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300",
                                                    "{format_ms(row.avg_ms)}"
                                                }
                                                div { class: "text-right tabular-nums font-semibold text-blue-600 dark:text-blue-400",
                                                    "{format_ms(row.p95_ms)}"
                                                }
                                                div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300",
                                                    "{format_ms(row.p99_ms)}"
                                                }
                                                div { class: "text-right tabular-nums font-semibold text-amber-700 dark:text-amber-400",
                                                    "{format_ms(row.max_ms)}"
                                                }
                                                div { class: "{err_class}",
                                                    "{format_num(row.error_count)}"
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
}

#[component]
fn ApiSortHeader(
    label: String,
    col_key: ApiSortKey,
    current_key: ApiSortKey,
    current_dir: SortDirection,
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
                if f.sort_key == col_key {
                    f.sort_dir = if f.sort_dir == SortDirection::Asc {
                        SortDirection::Desc
                    } else {
                        SortDirection::Asc
                    };
                } else {
                    f.sort_key = col_key;
                    f.sort_dir = SortDirection::Desc;
                }
                crate::store::set_filters(store, f);
            },
            class: cn(&[
                "group flex w-full items-center gap-1 cursor-pointer select-none transition-colors border-0 bg-transparent justify-end text-right",
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
