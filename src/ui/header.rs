//! `AppHeader` — port of `src/components/AppHeader.tsx`.

use dioxus::prelude::*;

use crate::store::{
    clear, show_toast, toggle_theme, use_analysis_store, use_app_mode_store, AppMode, SourceKind,
};
use crate::ui::icons::Icon;
use crate::utils::cn::cn;
use crate::utils::format::{format_bytes, format_date};

fn build_date_range_badge(dates: &[String]) -> Option<String> {
    if dates.is_empty() {
        return None;
    }
    if dates.len() > 1 {
        return Some(format!(
            "{} → {} ({} days)",
            format_date(dates.first().map(|s| s.as_str())),
            format_date(dates.last().map(|s| s.as_str())),
            dates.len()
        ));
    }
    Some(format_date(dates.first().map(|s| s.as_str())))
}

fn build_source_label(kind: SourceKind, file_name: Option<&str>, file_size: Option<u64>) -> Option<String> {
    match kind {
        SourceKind::File => {
            let name = file_name?;
            match file_size {
                Some(size) => Some(format!("{name} · {}", format_bytes(size))),
                None => Some(name.to_string()),
            }
        }
        SourceKind::Paste => Some("Pasted text".to_string()),
        SourceKind::None => None,
    }
}

#[component]
pub fn AppHeader() -> Element {
    let store = use_analysis_store();
    let app = use_app_mode_store();

    let is_mongo = app.mode() == AppMode::Mongo;
    let is_dark = store.is_dark();
    let has_pm2_data = store.has_data();
    let can_pm2_export = has_pm2_data && !store.is_parsing();
    let can_pm2_clear = has_pm2_data || store.source_kind() != SourceKind::None;
    let date_badge = store
        .result()
        .and_then(|r| build_date_range_badge(&r.dates));
    let source_label = build_source_label(
        store.source_kind(),
        store.file_name().as_deref(),
        store.file_size(),
    );
    let file_names = store.file_names();
    let file_names_title = if file_names.len() > 1 {
        Some(file_names.join("\n"))
    } else {
        None
    };

    let has_mongo_data = false;

    rsx! {
        header { class: "sticky top-0 z-30 border-b border-slate-200 bg-white/95 backdrop-blur-md dark:border-slate-800 dark:bg-slate-900/95",
            div { class: "mx-auto flex max-w-7xl items-center justify-between gap-4 px-4 py-2.5",
                div { class: "min-w-0",
                    if is_mongo {
                        crate::ui::mongo::MongoHeaderInfo {}
                    } else {
                        div {
                            div { class: "flex items-center gap-2",
                                Icon { name: "file-text", class: "size-5 shrink-0 text-blue-600 dark:text-blue-400" }
                                h1 { class: "truncate text-base font-semibold tracking-tight text-slate-900 dark:text-slate-100",
                                    "PM2 Log Analyzer"
                                }
                                if let Some(badge) = date_badge {
                                    span { class: "inline-flex items-center rounded-full bg-blue-50 px-2 py-0.5 text-[11px] font-medium text-blue-700 dark:bg-blue-950/60 dark:text-blue-300",
                                        "{badge}"
                                    }
                                }
                            }
                            if let Some(label) = source_label {
                                p {
                                    class: "mt-0.5 truncate font-mono-data text-xs text-slate-500 dark:text-slate-400",
                                    title: file_names_title,
                                    "{label}"
                                }
                            } else {
                                p { class: "mt-0.5 text-xs text-slate-500 dark:text-slate-400",
                                    "API latency & cron insight"
                                }
                            }
                        }
                    }
                }

                div { class: "flex shrink-0 items-center rounded-xl bg-slate-100 p-1 dark:bg-slate-800",
                    button {
                        r#type: "button",
                        "data-testid": "app-switcher-pm2",
                        onclick: move |_| crate::store::set_mode(app, AppMode::Pm2),
                        class: cn(&[
                            "relative flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold transition-all cursor-pointer border-0",
                            if !is_mongo {
                                "bg-white text-blue-600 shadow-xs dark:bg-slate-900 dark:text-blue-400"
                            } else {
                                "text-slate-600 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200"
                            },
                        ]),
                        Icon { name: "file-text", class: "size-3.5" }
                        span { "PM2 Logs" }
                        if has_pm2_data && is_mongo {
                            span {
                                class: "size-1.5 rounded-full bg-blue-500 animate-pulse",
                                title: "PM2 logs loaded",
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        "data-testid": "app-switcher-mongo",
                        onclick: move |_| crate::store::set_mode(app, AppMode::Mongo),
                        class: cn(&[
                            "relative flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold transition-all cursor-pointer border-0",
                            if is_mongo {
                                "bg-white text-emerald-600 shadow-xs dark:bg-slate-900 dark:text-emerald-400"
                            } else {
                                "text-slate-600 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200"
                            },
                        ]),
                        Icon { name: "database", class: "size-3.5" }
                        span { "MongoDB Logs" }
                        if has_mongo_data && !is_mongo {
                            span {
                                class: "size-1.5 rounded-full bg-emerald-500 animate-pulse",
                                title: "MongoDB logs loaded",
                            }
                        }
                    }
                }

                div { class: "flex shrink-0 items-center gap-2",
                    button {
                        r#type: "button",
                        onclick: move |_| toggle_theme(store),
                        title: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
                        "aria-label": if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
                        class: "inline-flex items-center justify-center rounded-lg border border-slate-200 bg-white p-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700 cursor-pointer",
                        if is_dark {
                            Icon { name: "sun", class: "size-4 text-amber-400" }
                        } else {
                            Icon { name: "moon", class: "size-4 text-slate-600" }
                        }
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| {
                            if is_mongo {
                                crate::utils::export_spreadsheet::export_mongo_spreadsheet_placeholder(store);
                            } else {
                                crate::utils::export_spreadsheet::export_spreadsheet_data(store);
                            }
                        },
                        disabled: !(if is_mongo { false } else { can_pm2_export }),
                        class: "inline-flex items-center gap-1.5 rounded-lg border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700 cursor-pointer",
                        Icon { name: "download", class: "size-3.5" }
                        "Export"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| {
                            if is_mongo {
                                show_toast(store, "MongoDB clear not wired yet");
                            } else {
                                clear(store);
                            }
                        },
                        disabled: !(if is_mongo { has_mongo_data } else { can_pm2_clear }),
                        class: "inline-flex items-center gap-1.5 rounded-lg border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700 cursor-pointer",
                        Icon { name: "eraser", class: "size-3.5" }
                        "Clear"
                    }
                }
            }
        }
    }
}
