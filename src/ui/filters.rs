use dioxus::prelude::*;
use crate::core::models::{FilterOptions, Method, PathNormMode, StatusFamily};

#[component]
pub fn FilterBar(
    has_data: Signal<bool>,
    filters: Signal<FilterOptions>,
    sort_key: Signal<String>,
    top_n: Signal<usize>,
    selected_methods: Signal<Vec<String>>,
    on_filter_change: EventHandler<()>,
) -> Element {
    if !has_data() {
        return rsx! {};
    }

    let current_norm = match filters().path_norm_mode {
        PathNormMode::CollapseIds => "collapseIds",
        PathNormMode::StripQuery => "stripQuery",
        PathNormMode::Raw => "exact",
    };

    let current_status = match filters().status_family {
        StatusFamily::All => "all",
        StatusFamily::Success => "2xx",
        StatusFamily::Redirect => "3xx",
        StatusFamily::ClientError => "4xx",
        StatusFamily::ServerError => "5xx",
        StatusFamily::ErrorOnly => "errors",
    };

    let all_methods = vec!["DELETE", "GET", "HEAD", "PATCH", "POST", "PUT"];
    let selected_set: Vec<String> = selected_methods();
    let is_all_selected = selected_set.is_empty();

    let field_class = "rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-800 focus:border-blue-500 dark:border-slate-700 dark:bg-slate-950 dark:text-slate-100 dark:focus:border-blue-400";

    rsx! {
        section { class: "rounded border border-slate-200 bg-white px-3 py-3 dark:border-slate-800 dark:bg-slate-900",
            div { class: "flex flex-wrap items-end gap-3",
                label { class: "block",
                    span { class: "mb-1 block text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Normalize"
                    }
                    select {
                        value: "{current_norm}",
                        onchange: move |e: Event<FormData>| {
                            let mode = match e.value().as_str() {
                                "stripQuery" => PathNormMode::StripQuery,
                                "exact" => PathNormMode::Raw,
                                _ => PathNormMode::CollapseIds,
                            };
                            filters.with_mut(|f| f.path_norm_mode = mode);
                            on_filter_change.call(());
                        },
                        class: "{field_class}",
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
                        value: "{current_status}",
                        onchange: move |e: Event<FormData>| {
                            let status = match e.value().as_str() {
                                "2xx" => StatusFamily::Success,
                                "3xx" => StatusFamily::Redirect,
                                "4xx" => StatusFamily::ClientError,
                                "5xx" => StatusFamily::ServerError,
                                "errors" => StatusFamily::ErrorOnly,
                                _ => StatusFamily::All,
                            };
                            filters.with_mut(|f| f.status_family = status);
                            on_filter_change.call(());
                        },
                        class: "{field_class}",
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
                        r#type: "number",
                        min: "0",
                        value: "{filters().min_duration_ms}",
                        oninput: move |e: Event<FormData>| {
                            let ms = e.value().parse::<f32>().unwrap_or(0.0);
                            filters.with_mut(|f| f.min_duration_ms = ms);
                            on_filter_change.call(());
                        },
                        class: "{field_class} w-20"
                    }
                }

                label { class: "block",
                    span { class: "mb-1 block text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Sort"
                    }
                    select {
                        value: "{sort_key}",
                        onchange: move |e: Event<FormData>| {
                            sort_key.set(e.value());
                            on_filter_change.call(());
                        },
                        class: "{field_class}",
                        option { value: "p95Ms", "p95" }
                        option { value: "p99Ms", "p99" }
                        option { value: "avgMs", "avg" }
                        option { value: "maxMs", "max" }
                        option { value: "count", "count" }
                        option { value: "errorCount", "errors" }
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
                        value: "{top_n}",
                        oninput: move |e: Event<FormData>| {
                            let val = e.value().parse::<usize>().unwrap_or(50).clamp(1, 500);
                            top_n.set(val);
                            on_filter_change.call(());
                        },
                        class: "{field_class} w-20"
                    }
                }

                label { class: "block min-w-[12rem] flex-1",
                    span { class: "mb-1 block text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                        "Search"
                    }
                    input {
                        r#type: "search",
                        value: "{filters().search_query}",
                        oninput: move |e: Event<FormData>| {
                            filters.with_mut(|f| f.search_query = e.value());
                            on_filter_change.call(());
                        },
                        placeholder: "Filter endpoints… (/)",
                        class: "{field_class} w-full"
                    }
                }
            }

            div { class: "mt-3 flex flex-wrap items-center gap-1.5",
                span { class: "mr-1 text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400",
                    "Methods"
                }
                button {
                    r#type: "button",
                    onclick: move |_| {
                        selected_methods.set(Vec::new());
                        filters.with_mut(|f| f.method = None);
                        on_filter_change.call(());
                    },
                    class: if is_all_selected {
                        "rounded px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide ring-1 transition-colors bg-blue-50 text-blue-700 ring-blue-200 dark:bg-blue-950/60 dark:text-blue-300 dark:ring-blue-800 border-0 cursor-pointer"
                    } else {
                        "rounded px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide ring-1 transition-colors bg-slate-50 text-slate-400 ring-slate-200 dark:bg-slate-800/60 dark:text-slate-400 dark:ring-slate-700 border-0 cursor-pointer"
                    },
                    "All"
                }
                for m in all_methods.iter() {
                    {
                        let m_str = m.to_string();
                        let is_active = is_all_selected || selected_set.contains(&m_str);
                        let m_val = match *m {
                            "GET" => Method::Get,
                            "POST" => Method::Post,
                            "PUT" => Method::Put,
                            "PATCH" => Method::Patch,
                            "DELETE" => Method::Delete,
                            _ => Method::Head,
                        };

                        let m_label = m_str.clone();
                        rsx! {
                            button {
                                key: "{m_str}",
                                r#type: "button",
                                onclick: move |_| {
                                    if is_all_selected {
                                        selected_methods.set(vec![m_label.clone()]);
                                        filters.with_mut(|f| f.method = Some(m_val));
                                    } else {
                                        let mut cur = selected_methods();
                                        if cur.contains(&m_label) {
                                            cur.retain(|x| x != &m_label);
                                        } else {
                                            cur.push(m_label.clone());
                                        }
                                        selected_methods.set(cur);
                                        filters.with_mut(|f| f.method = Some(m_val));
                                    }
                                    on_filter_change.call(());
                                },
                                class: if is_active {
                                    "rounded px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide ring-1 transition-colors bg-blue-50 text-blue-700 ring-blue-200 dark:bg-blue-950/60 dark:text-blue-300 dark:ring-blue-800 border-0 cursor-pointer"
                                } else {
                                    "rounded px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide ring-1 transition-colors bg-slate-50 text-slate-400 ring-slate-200 dark:bg-slate-800/60 dark:text-slate-400 dark:ring-slate-700 border-0 cursor-pointer"
                                },
                                "{m_str}"
                            }
                        }
                    }
                }
                if !is_all_selected {
                    button {
                        r#type: "button",
                        onclick: move |_| {
                            selected_methods.set(Vec::new());
                            filters.with_mut(|f| f.method = None);
                            on_filter_change.call(());
                        },
                        class: "text-[11px] text-slate-500 underline-offset-2 hover:underline dark:text-slate-400 bg-transparent border-0 cursor-pointer",
                        "Reset"
                    }
                }
            }
        }
    }
}
