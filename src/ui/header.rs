use dioxus::prelude::*;
use crate::utils::exporter::{export_to_csv, export_to_excel, export_to_json};
use crate::core::models::LogAnalysisSummary;
use rfd::FileDialog;

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[component]
pub fn AppHeader(
    theme: Signal<String>,
    source_kind: Signal<String>,
    file_name: Signal<Option<String>>,
    file_size: Signal<u64>,
    has_data: Signal<bool>,
    is_parsing: Signal<bool>,
    summary: Signal<LogAnalysisSummary>,
    on_toast: EventHandler<String>,
    on_clear: EventHandler<()>,
) -> Element {
    let (is_mono, subtitle) = if source_kind() == "file" && file_name().is_some() {
        let name = file_name().unwrap();
        let size_str = format_bytes(file_size());
        (true, format!("{} · {}", name, size_str))
    } else if source_kind() == "paste" {
        (true, "Pasted text".to_string())
    } else {
        (false, "API latency & cron insight".to_string())
    };

    let is_dark = theme() == "dark";

    let handle_export = move |_| {
        if !has_data() {
            on_toast.call("Nothing to export yet".to_string());
            return;
        }

        if let Some(path) = FileDialog::new()
            .set_file_name("pm2_analysis.xlsx")
            .add_filter("Excel Workbook", &["xlsx"])
            .add_filter("CSV File", &["csv"])
            .add_filter("JSON File", &["json"])
            .save_file()
        {
            let path_str = path.to_string_lossy();
            let res = if path_str.ends_with(".csv") {
                export_to_csv(&summary(), &path_str).map_err(|e| e.to_string())
            } else if path_str.ends_with(".json") {
                export_to_json(&summary(), &path_str).map_err(|e| e.to_string())
            } else {
                export_to_excel(&summary(), &path_str).map_err(|e| e.to_string())
            };

            match res {
                Ok(_) => on_toast.call(format!("Exported to {}", path.file_name().unwrap_or_default().to_string_lossy())),
                Err(e) => on_toast.call(format!("Export failed: {}", e)),
            }
        }
    };

    let toggle_theme = move |_| {
        if is_dark {
            theme.set("light".to_string());
        } else {
            theme.set("dark".to_string());
        }
    };

    rsx! {
        header { class: "sticky top-0 z-20 border-b border-slate-200 bg-white/95 backdrop-blur dark:border-slate-800 dark:bg-slate-900/95",
            div { class: "mx-auto flex max-w-7xl items-center justify-between gap-4 px-4 py-3",
                div { class: "min-w-0",
                    div { class: "flex items-center gap-2",
                        svg {
                            width: "20",
                            height: "20",
                            class: "size-5 shrink-0 text-blue-600 dark:text-blue-400",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            view_box: "0 0 24 24",
                            path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
                            polyline { points: "14 2 14 8 20 8" }
                            line { x1: "16", y1: "13", x2: "8", y2: "13" }
                            line { x1: "16", y1: "17", x2: "8", y2: "17" }
                            polyline { points: "10 9 9 9 8 9" }
                        }
                        h1 { class: "truncate text-base font-semibold tracking-tight text-slate-900 dark:text-slate-100",
                            "PM2 Log Analyzer"
                        }
                    }
                    p {
                        class: if is_mono { "mt-0.5 truncate font-mono-data text-xs text-slate-500 dark:text-slate-400" } else { "mt-0.5 text-xs text-slate-500 dark:text-slate-400" },
                        "{subtitle}"
                    }
                }
                div { class: "flex shrink-0 items-center gap-2",
                    button {
                        r#type: "button",
                        onclick: toggle_theme,
                        title: if is_dark { "Switch to light mode" } else { "Switch to dark mode" },
                        class: "inline-flex items-center justify-center rounded border border-slate-200 bg-white p-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700 cursor-pointer",
                        if is_dark {
                            svg { width: "16", height: "16", class: "size-4 text-amber-400", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                                circle { cx: "12", cy: "12", r: "5" }
                                line { x1: "12", y1: "1", x2: "12", y2: "3" }
                                line { x1: "12", y1: "21", x2: "12", y2: "23" }
                                line { x1: "4.22", y1: "4.22", x2: "5.64", y2: "5.64" }
                                line { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }
                                line { x1: "1", y1: "12", x2: "3", y2: "12" }
                                line { x1: "21", y1: "12", x2: "23", y2: "12" }
                                line { x1: "4.22", y1: "19.78", x2: "5.64", y2: "18.36" }
                                line { x1: "18.36", y1: "5.64", x2: "19.78", y2: "4.22" }
                            }
                        } else {
                            svg { width: "16", height: "16", class: "size-4 text-slate-600", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                                path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        onclick: handle_export,
                        disabled: !has_data() || is_parsing(),
                        class: "inline-flex items-center gap-1.5 rounded border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700",
                        svg { width: "14", height: "14", class: "size-3.5", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                            polyline { points: "7 10 12 15 17 10" }
                            line { x1: "12", y1: "15", x2: "12", y2: "3" }
                        }
                        "Export"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| on_clear.call(()),
                        disabled: !has_data() && source_kind() == "none",
                        class: "inline-flex items-center gap-1.5 rounded border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700",
                        svg { width: "14", height: "14", class: "size-3.5", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            path { d: "m7 21-4-4 8-8 4 4-8 8Z" }
                            path { d: "M17 11l4-4-4-4-4 4 4 4Z" }
                        }
                        "Clear"
                    }
                }
            }
        }
    }
}
