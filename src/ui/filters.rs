//! `FilterBar` — port of `src/components/FilterBar.tsx`.

use dioxus::prelude::*;

use crate::core::models::{NormalizeMode, StatusFamily};
use crate::store::{
    count_active_analysis_filters, reaggregate, reset_filters, set_filters, use_analysis_store,
    AnalysisFilters, ApiSortKey, SortDirection,
};
use crate::utils::cn::cn;
use crate::utils::format::format_date;

const FIELD_CLASS: &str = "rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-800 focus:border-blue-500 dark:border-slate-700 dark:bg-slate-950 dark:text-slate-100 dark:focus:border-blue-400";

pub const FILTER_SEARCH_KEY_SCRIPT: &str = r#"
window.addEventListener('keydown', function (e) {
  if (e.key !== '/' || e.metaKey || e.ctrlKey || e.altKey) return;
  const t = e.target;
  if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
  const input = document.querySelector('input[data-filter-search]');
  if (input) { e.preventDefault(); input.focus(); }
});
"#;

#[component]
pub fn FilterBar() -> Element {
    let store = use_analysis_store();
    if !store.has_data() {
        return rsx! {};
    }
    let filters = store.filters();
    let active_count = count_active_analysis_filters(&filters);
    let methods = store.result().map(|r| r.methods).unwrap_or_default();
    let dates = store.result().map(|r| r.dates).unwrap_or_default();
    let all_selected = filters.methods.is_empty();

    let apply = move |updated: AnalysisFilters, refresh: bool| {
        set_filters(store, updated);
        if refresh {
            reaggregate(store);
        }
    };

    rsx! {
        section {
            "data-ui-id": "filter-bar",
            class: "rounded border border-slate-200 bg-white px-3 py-3 dark:border-slate-800 dark:bg-slate-900",
            div { class: "flex flex-wrap items-end gap-3",
                label { class: "block min-w-[14rem] flex-1",
                    span { class: "mb-1 block text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Search"
                    }
                    input {
                        "data-filter-search": "true",
                        r#type: "search",
                        value: "{filters.query}",
                        oninput: move |e: Event<FormData>| {
                            let mut f = store.filters();
                            f.query = e.value();
                            apply(f, false);
                        },
                        placeholder: "Filter endpoints… (/)",
                        class: cn(&[FIELD_CLASS, "w-full"]),
                    }
                }
                label { class: "block",
                    span { class: "mb-1 block text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Normalize"
                    }
                    select {
                        value: "{filters.normalize_mode.key()}",
                        onchange: move |e: Event<FormData>| {
                            if let Some(mode) = NormalizeMode::from_key(&e.value()) {
                                let mut f = store.filters();
                                f.normalize_mode = mode;
                                apply(f, true);
                            }
                        },
                        class: "{FIELD_CLASS}",
                        option { value: "collapseIds", "Collapse IDs" }
                        option { value: "stripQuery", "Strip query" }
                        option { value: "exact", "Exact path" }
                    }
                }
                label { class: "block",
                    span { class: "mb-1 block text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Status"
                    }
                    select {
                        "data-testid": "filter-status",
                        value: "{filters.status_family.key()}",
                        onchange: move |e: Event<FormData>| {
                            if let Some(family) = StatusFamily::from_key(&e.value()) {
                                let mut f = store.filters();
                                f.status_family = family;
                                apply(f, true);
                            }
                        },
                        class: "{FIELD_CLASS}",
                        option { value: "all", "All" }
                        option { value: "2xx", "2xx" }
                        option { value: "3xx", "3xx" }
                        option { value: "4xx", "4xx" }
                        option { value: "5xx", "5xx" }
                    }
                }
                label { class: "block",
                    span { class: "mb-1 block text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Min ms"
                    }
                    input {
                        "data-testid": "filter-min-ms",
                        r#type: "number",
                        min: "0",
                        value: "{fmt_min_ms(filters.min_ms)}",
                        oninput: move |e: Event<FormData>| {
                            let mut f = store.filters();
                            f.min_ms = e.value().parse::<f64>().unwrap_or(0.0).max(0.0);
                            apply(f, true);
                        },
                        class: cn(&[FIELD_CLASS, "w-20"]),
                    }
                }
                label { class: "block",
                    span { class: "mb-1 block text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Sort"
                    }
                    select {
                        value: "{api_sort_key_str(filters.sort_key)}",
                        onchange: move |e: Event<FormData>| {
                            if let Some(key) = parse_api_sort_key(&e.value()) {
                                let mut f = store.filters();
                                f.sort_key = key;
                                apply(f, false);
                            }
                        },
                        class: "{FIELD_CLASS}",
                        option { value: "p95Ms", "p95" }
                        option { value: "p99Ms", "p99" }
                        option { value: "avgMs", "avg" }
                        option { value: "maxMs", "max" }
                        option { value: "count", "count" }
                        option { value: "errorCount", "errors" }
                        option { value: "path", "endpoint" }
                    }
                }
                label { class: "block",
                    span { class: "mb-1 block text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Top N"
                    }
                    input {
                        r#type: "number",
                        min: "1",
                        max: "500",
                        value: "{filters.top_n}",
                        oninput: move |e: Event<FormData>| {
                            let mut f = store.filters();
                            f.top_n = e
                                .value()
                                .parse::<usize>()
                                .unwrap_or(50)
                                .clamp(1, 500);
                            apply(f, false);
                        },
                        class: cn(&[FIELD_CLASS, "w-20"]),
                    }
                }
                div { class: "ml-auto flex items-center",
                    button {
                        r#type: "button",
                        "data-testid": "pm2-reset-filters",
                        disabled: active_count == 0,
                        onclick: move |_| reset_filters(store),
                        class: cn(&[
                            "flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-semibold transition-all cursor-pointer",
                            if active_count > 0 {
                                "border border-rose-200 bg-rose-50 text-rose-700 hover:bg-rose-100 hover:text-rose-800 dark:border-rose-900/60 dark:bg-rose-950/40 dark:text-rose-300 dark:hover:bg-rose-900/60"
                            } else {
                                "border border-transparent text-slate-400 opacity-40 cursor-not-allowed dark:text-slate-500"
                            },
                        ]),
                        title: if active_count > 0 { "Reset all filters to defaults" } else { "No active filters to reset" },
                        Icon18 { name: "rotate-ccw" }
                        span { "Reset all filters" }
                        if active_count > 0 {
                            span { class: "rounded-full bg-rose-200/80 px-1.5 py-0.2 text-[10px] font-bold text-rose-800 dark:bg-rose-900 dark:text-rose-200",
                                "{active_count}"
                            }
                        }
                    }
                }
            }

            if dates.len() > 1 || !methods.is_empty() {
                div { class: "mt-2.5 flex flex-wrap items-center gap-3",
                    if dates.len() > 1 {
                        div {
                            "data-testid": "filter-date",
                            class: "flex flex-wrap items-center gap-1.5",
                            span { class: "mr-1 text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                                "Day"
                            }
                            button {
                                r#type: "button",
                                onclick: move |_| {
                                    let mut f = store.filters();
                                    f.date_filter = "all".to_string();
                                    apply(f, true);
                                },
                                class: cn(&[
                                    "rounded px-2 py-0.5 text-[10px] font-medium tracking-wide ring-1 transition-colors cursor-pointer border-0",
                                    if filters.date_filter == "all" {
                                        "bg-blue-50 text-blue-700 ring-blue-200 dark:bg-blue-950/60 dark:text-blue-300 dark:ring-blue-800"
                                    } else {
                                        "bg-slate-50 text-slate-500 ring-slate-200 hover:bg-slate-100 dark:bg-slate-800/60 dark:text-slate-400 dark:ring-slate-700 dark:hover:bg-slate-800"
                                    },
                                ]),
                                "All Days ({dates.len()})"
                            }
                            for date in dates.iter() {
                                {
                                    let d = date.clone();
                                    let d_for_key = d.clone();
                                    let active = filters.date_filter == *date;
                                    rsx! {
                                        button {
                                            key: "{d_for_key}",
                                            r#type: "button",
                                            onclick: move |_| {
                                                let mut f = store.filters();
                                                f.date_filter = d.clone();
                                                apply(f, true);
                                            },
                                            class: cn(&[
                                                "rounded px-2 py-0.5 font-mono-data text-[10px] tracking-wide ring-1 transition-colors cursor-pointer border-0",
                                                if active {
                                                    "bg-blue-50 text-blue-700 ring-blue-200 dark:bg-blue-950/60 dark:text-blue-300 dark:ring-blue-800"
                                                } else {
                                                    "bg-slate-50 text-slate-500 ring-slate-200 hover:bg-slate-100 dark:bg-slate-800/60 dark:text-slate-400 dark:ring-slate-700 dark:hover:bg-slate-800"
                                                },
                                            ]),
                                            "{format_date(Some(&d))}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if dates.len() > 1 && !methods.is_empty() {
                        div { class: "hidden h-4 w-px bg-slate-200 sm:block dark:bg-slate-700" }
                    }
                    if !methods.is_empty() {
                        div { class: "flex flex-wrap items-center gap-1.5",
                            span { class: "mr-1 text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                                "Methods"
                            }
                            MethodChip { label: "All".to_string(), method: None, all_selected }
                            for method in methods.iter() {
                                MethodChip {
                                    key: "{method}",
                                    label: method.clone(),
                                    method: Some(method.clone()),
                                    all_selected,
                                }
                            }
                            if !all_selected {
                                button {
                                    r#type: "button",
                                    onclick: move |_| {
                                        let mut f = store.filters();
                                        f.methods = Vec::new();
                                        apply(f, false);
                                    },
                                    class: "text-[11px] text-slate-500 underline-offset-2 hover:underline dark:text-slate-400 cursor-pointer border-0 bg-transparent",
                                    "Reset"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn fmt_min_ms(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

fn api_sort_key_str(key: ApiSortKey) -> &'static str {
    match key {
        ApiSortKey::P95Ms => "p95Ms",
        ApiSortKey::P99Ms => "p99Ms",
        ApiSortKey::AvgMs => "avgMs",
        ApiSortKey::MaxMs => "maxMs",
        ApiSortKey::Count => "count",
        ApiSortKey::ErrorCount => "errorCount",
        ApiSortKey::Path => "path",
    }
}

fn parse_api_sort_key(value: &str) -> Option<ApiSortKey> {
    match value {
        "p95Ms" => Some(ApiSortKey::P95Ms),
        "p99Ms" => Some(ApiSortKey::P99Ms),
        "avgMs" => Some(ApiSortKey::AvgMs),
        "maxMs" => Some(ApiSortKey::MaxMs),
        "count" => Some(ApiSortKey::Count),
        "errorCount" => Some(ApiSortKey::ErrorCount),
        "path" => Some(ApiSortKey::Path),
        _ => None,
    }
}

#[component]
fn Icon18(name: String) -> Element {
    rsx! {
        crate::ui::icons::Icon { name, class: "size-3" }
    }
}

#[component]
fn MethodChip(
    label: String,
    method: Option<String>,
    all_selected: bool,
) -> Element {
    let store = use_analysis_store();
    let is_selected = method
        .as_deref()
        .map(|m| store.filters().methods.iter().any(|x| x == m))
        .unwrap_or(false);
    let active = all_selected || is_selected;
    let method_for_click = method.clone();
    rsx! {
        button {
            r#type: "button",
            onclick: move |_| {
                let mut f = store.filters();
                match &method_for_click {
                    None => f.methods = Vec::new(),
                    Some(m) => {
                        if all_selected {
                            f.methods = vec![m.clone()];
                        } else if f.methods.iter().any(|x| x == m) {
                            f.methods.retain(|x| x != m);
                        } else {
                            f.methods.push(m.clone());
                        }
                    }
                }
                set_filters(store, f);
            },
            class: cn(&[
                "rounded px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide ring-1 transition-colors cursor-pointer border-0",
                if active {
                    "bg-blue-50 text-blue-700 ring-blue-200 dark:bg-blue-950/60 dark:text-blue-300 dark:ring-blue-800"
                } else {
                    "bg-slate-50 text-slate-400 ring-slate-200 dark:bg-slate-800/60 dark:text-slate-400 dark:ring-slate-700"
                },
            ]),
            "{label}"
        }
    }
}

pub const _SORT_DIR_UNUSED: SortDirection = SortDirection::Desc;
