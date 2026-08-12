use dioxus::prelude::*;

#[component]
pub fn IngestPanel(
    has_data: Signal<bool>,
    is_parsing: Signal<bool>,
    progress_percent: Signal<u32>,
    on_file_select: EventHandler<std::path::PathBuf>,
    on_paste_analyze: EventHandler<String>,
    on_toast: EventHandler<String>,
) -> Element {
    let mut paste_open = use_signal(|| false);
    let mut paste_text = use_signal(|| String::new());

    let handle_file_picker = move |_| {
        if is_parsing() {
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Log files", &["log", "txt"])
            .pick_file()
        {
            on_file_select.call(path);
        }
    };

    let handle_analyze_click = move |_| {
        let text = paste_text().trim().to_string();
        if text.is_empty() {
            on_toast.call("Paste some log lines first".to_string());
            return;
        }
        on_paste_analyze.call(text);
    };

    rsx! {
        section { class: "rounded border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900",
            div { class: "flex flex-col items-center justify-center gap-3 px-6 py-10 text-center transition-colors bg-white dark:bg-slate-900",
                svg {
                    width: "32",
                    height: "32",
                    class: "size-8 text-slate-400 dark:text-slate-500",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    view_box: "0 0 24 24",
                    path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                    polyline { points: "17 8 12 3 7 8" }
                    line { x1: "12", y1: "3", x2: "12", y2: "15" }
                }
                div {
                    p { class: "text-sm font-medium text-slate-800 dark:text-slate-200",
                        if has_data() { "Drop a new file to replace" } else { "Drop a PM2 log file" }
                    }
                    p { class: "mt-1 text-xs text-slate-500 dark:text-slate-400",
                        ".log / .txt — streamed off the main thread for large dumps"
                    }
                }
                div { class: "flex flex-wrap items-center justify-center gap-2",
                    button {
                        r#type: "button",
                        disabled: is_parsing(),
                        onclick: handle_file_picker,
                        class: "rounded bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700 disabled:opacity-40 cursor-pointer border-0",
                        "Browse files"
                    }
                    button {
                        r#type: "button",
                        disabled: is_parsing(),
                        onclick: move |_| paste_open.set(!paste_open()),
                        class: "inline-flex items-center gap-1.5 rounded border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700 cursor-pointer",
                        svg { width: "14", height: "14", class: "size-3.5", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            path { d: "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" }
                            rect { x: "8", y: "2", width: "8", height: "4", rx: "1", ry: "1" }
                        }
                        "Paste logs"
                    }
                }
                if is_parsing() {
                    div { class: "w-full max-w-md",
                        div { class: "mb-1 flex justify-between text-[11px] text-slate-500 dark:text-slate-400",
                            span { class: "capitalize", "parsing" }
                            span { "{progress_percent()}%" }
                        }
                        div { class: "h-1.5 overflow-hidden rounded-full bg-slate-100 dark:bg-slate-800",
                            div {
                                class: "h-full bg-blue-600 transition-[width] duration-150",
                                style: "width: {progress_percent()}%"
                            }
                        }
                    }
                }
                if !has_data() && !is_parsing() {
                    p { class: "max-w-lg font-mono-data text-[11px] leading-relaxed text-slate-400 dark:text-slate-500",
                        "Example: 2026-07-24T00:00:10: GET /api/health 200 12.5 ms - 42"
                    }
                }
            }

            if paste_open() {
                div { class: "border-t border-slate-200 px-4 py-3 dark:border-slate-800",
                    label { class: "mb-1.5 block text-xs font-medium text-slate-600 dark:text-slate-300",
                        "Paste log lines (not persisted; max ~8 MB)"
                    }
                    textarea {
                        value: "{paste_text}",
                        oninput: move |e: Event<FormData>| paste_text.set(e.value()),
                        disabled: is_parsing(),
                        rows: "6",
                        placeholder: "Paste PM2 stdout/stderr here…",
                        class: "w-full resize-y rounded border border-slate-200 bg-slate-50 px-3 py-2 font-mono-data text-xs text-slate-800 placeholder:text-slate-400 focus:border-blue-500 focus:bg-white dark:border-slate-700 dark:bg-slate-950 dark:text-slate-100 dark:placeholder:text-slate-500 dark:focus:bg-slate-900"
                    }
                    div { class: "mt-2 flex justify-end",
                        button {
                            r#type: "button",
                            disabled: is_parsing(),
                            onclick: handle_analyze_click,
                            class: "rounded bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700 disabled:opacity-40 cursor-pointer border-0",
                            "Analyze paste"
                        }
                    }
                }
            }
        }
    }
}
