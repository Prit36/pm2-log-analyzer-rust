//! `IngestPanel` — port of `src/components/IngestPanel.tsx`.

use dioxus::html::HasFileData;
use dioxus::prelude::*;

use crate::core::pm2::LoadedSource;
use crate::store::{
    cancel, handle_log_files_upload, parse_text, set_source_paste, show_toast, use_analysis_store,
    AnalysisStore, PASTE_WARN_BYTES,
};
use crate::ui::icons::Icon;
use crate::utils::cn::cn;
use crate::utils::format::format_bytes;

const ACCEPT_EXTENSIONS: &[&str] = &["log", "txt", "out", "err", "zip", "gz", "json"];

pub fn is_valid_file_path(path: &std::path::Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if name.ends_with(".zip") || name.ends_with(".gz") {
        return true;
    }
    let ext = name.rsplit('.').next().unwrap_or("");
    if ACCEPT_EXTENSIONS.contains(&ext) {
        return true;
    }
    // `.log.1`, `.log.2`, … and bare numeric suffixes.
    if name.split('.').count() >= 2 {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() >= 2 && parts[parts.len() - 2] == "log" {
            return true;
        }
        if name.rsplit('.').next().is_some_and(|e| e.chars().all(|c| c.is_ascii_digit())) {
            return true;
        }
    }
    false
}

fn pick_files() -> Vec<LoadedSource> {
    let Some(paths) = rfd::FileDialog::new()
        .add_filter("Log files", &["log", "txt", "out", "err", "gz", "zip", "json", "1", "2", "3"])
        .pick_files()
    else {
        return Vec::new();
    };
    paths
        .into_iter()
        .filter(|p| is_valid_file_path(p))
        .map(LoadedSource::Path)
        .collect()
}

fn busy(store: AnalysisStore) -> bool {
    store.is_parsing()
}

#[component]
pub fn IngestPanel() -> Element {
    let mut store = use_analysis_store();
    let mut drag_over = use_signal(|| false);
    let mut pending_drop = use_signal(|| None::<Vec<LoadedSource>>);
    let mut paste_text = use_signal(String::new);

    let is_parsing = store.is_parsing();
    let has_data = store.has_data();
    let loaded_files = store.loaded_files();
    let paste_open = store.paste_open();
    let progress = store.progress();
    let is_busy = busy(store);

    let mut execute_append = move |files: Vec<LoadedSource>| {
        pending_drop.set(None);
        handle_log_files_upload(store, files, true);
    };
    let mut execute_replace = move |files: Vec<LoadedSource>| {
        pending_drop.set(None);
        handle_log_files_upload(store, files, false);
    };

    let mut open_picker = move |append: bool| {
        if busy(store) {
            return;
        }
        let files = pick_files();
        if files.is_empty() {
            return;
        }
        if append && store.has_data() && !store.loaded_files().is_empty() {
            execute_append(files);
        } else {
            execute_replace(files);
        }
    };

    let on_drop = move |event: Event<DragData>| {
        drag_over.set(false);
        if busy(store) {
            return;
        }
        let Some(files) = event.data().files() else {
            return;
        };
        let sources: Vec<LoadedSource> = files
            .files()
            .into_iter()
            .map(std::path::PathBuf::from)
            .filter(|p| is_valid_file_path(p))
            .map(LoadedSource::Path)
            .collect();
        if sources.is_empty() {
            show_toast(
                store,
                "Please upload log, text, or archive files (.log, .zip, .gz, .txt, etc.)",
            );
            return;
        }
        if store.has_data() && !store.loaded_files().is_empty() {
            pending_drop.set(Some(sources));
        } else {
            execute_replace(sources);
        }
    };

    let analyze_paste = move |_| {
        let text = paste_text().trim().to_string();
        if text.is_empty() {
            show_toast(store, "Paste some log lines first");
            return;
        }
        let bytes = text.len();
        if bytes > PASTE_WARN_BYTES {
            show_toast(
                store,
                format!(
                    "Paste is {} — save as a .log file and upload instead (limit ~{})",
                    format_bytes(bytes as u64),
                    format_bytes(PASTE_WARN_BYTES as u64)
                ),
            );
            return;
        }
        set_source_paste(store);
        parse_text(store, text);
    };

    let drag_class = if drag_over() {
        if has_data {
            "bg-blue-50/80 dark:bg-blue-950/40"
        } else {
            "bg-blue-50 dark:bg-blue-950/40"
        }
    } else {
        "bg-white dark:bg-slate-900"
    };

    rsx! {
        section { "data-ui-id": "ingest-panel", class: "rounded border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900",
            if has_data {
                div {
                    class: cn(&[
                        "relative flex flex-wrap items-center justify-between gap-3 px-4 py-2.5 transition-colors",
                        drag_class,
                        if is_busy { "opacity-60" } else { "" },
                    ]),
                    ondragover: move |event: Event<DragData>| {
                        event.prevent_default();
                        if !busy(store) {
                            drag_over.set(true);
                        }
                    },
                    ondragleave: move |_| drag_over.set(false),
                    ondrop: on_drop,
                    div { class: "flex min-w-0 flex-1 flex-wrap items-center gap-2",
                        Icon { name: "upload", class: "size-4 shrink-0 text-slate-400 dark:text-slate-500" }
                        LoadedFilesDisplay { files: loaded_files.clone() }
                    }
                    div { class: "flex shrink-0 items-center gap-2",
                        if is_parsing {
                            div { class: "flex items-center gap-3",
                                if let Some(p) = progress.clone() {
                                    div { class: "flex items-center gap-2 font-mono-data text-xs text-slate-600 dark:text-slate-300",
                                        span { class: "capitalize", "{p.stage}" }
                                        span { class: "font-semibold text-blue-600 dark:text-blue-400",
                                            "{p.percent}%"
                                        }
                                    }
                                }
                                button {
                                    r#type: "button",
                                    onclick: move |_| cancel(store),
                                    class: "rounded border border-rose-200 bg-rose-50 px-2.5 py-1 text-xs font-medium text-rose-700 hover:bg-rose-100 dark:border-rose-900/50 dark:bg-rose-950/40 dark:text-rose-300 dark:hover:bg-rose-900/60 cursor-pointer",
                                    "Cancel"
                                }
                            }
                        } else {
                            button {
                                r#type: "button",
                                disabled: is_busy,
                                onclick: move |_| open_picker(true),
                                class: "inline-flex items-center gap-1.5 rounded bg-blue-600 px-2.5 py-1 text-xs font-medium text-white hover:bg-blue-700 disabled:opacity-40 cursor-pointer border-0",
                                Icon { name: "file-plus", class: "size-3.5" }
                                "Add / Append"
                            }
                            button {
                                r#type: "button",
                                disabled: is_busy,
                                onclick: move |_| open_picker(false),
                                class: "inline-flex items-center gap-1.5 rounded border border-slate-200 bg-white px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700 cursor-pointer",
                                Icon { name: "refresh-cw", class: "size-3.5" }
                                "Replace"
                            }
                            button {
                                r#type: "button",
                                disabled: is_busy,
                                onclick: move |_| store.paste_open.set(!store.paste_open()),
                                class: "inline-flex items-center gap-1.5 rounded border border-slate-200 bg-white px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700 cursor-pointer",
                                Icon { name: "clipboard-paste", class: "size-3.5" }
                                "Paste"
                            }
                        }
                    }
                    if is_parsing {
                        if let Some(p) = progress.clone() {
                            div { class: "absolute inset-x-0 bottom-0 h-0.5 overflow-hidden bg-slate-100 dark:bg-slate-800",
                                div {
                                    class: "h-full bg-blue-600 transition-[width] duration-150",
                                    style: "width: {p.percent}%",
                                }
                            }
                        }
                    }
                }
            } else {
                div {
                    class: cn(&[
                        "flex flex-col items-center justify-center gap-3 px-6 py-10 text-center transition-colors",
                        drag_class,
                        if is_busy { "opacity-60" } else { "" },
                    ]),
                    ondragover: move |event: Event<DragData>| {
                        event.prevent_default();
                        if !busy(store) {
                            drag_over.set(true);
                        }
                    },
                    ondragleave: move |_| drag_over.set(false),
                    ondrop: on_drop,
                    Icon { name: "upload", class: "size-8 text-slate-400 dark:text-slate-500" }
                    div {
                        p { class: "text-sm font-medium text-slate-800 dark:text-slate-200",
                            if is_parsing { "Parsing PM2 log file(s)…" } else { "Drop PM2 / API logs or .zip archive" }
                        }
                        p { class: "mt-1 text-xs text-slate-500 dark:text-slate-400",
                            ".log / .log.1 / .gz / .zip (auto-classifies API & MongoDB logs)"
                        }
                    }
                    if is_parsing {
                        div { class: "w-full max-w-md space-y-3",
                            if let Some(p) = progress.clone() {
                                div {
                                    div { class: "mb-1.5 flex justify-between text-[11px] text-slate-500 dark:text-slate-400",
                                        span { class: "capitalize", "{p.stage}" }
                                        span { class: "font-mono-data font-semibold text-blue-600 dark:text-blue-400",
                                            "{p.percent}%"
                                        }
                                    }
                                    div { class: "h-2 overflow-hidden rounded-full bg-slate-100 dark:bg-slate-800",
                                        div {
                                            class: "h-full bg-blue-600 transition-[width] duration-150",
                                            style: "width: {p.percent}%",
                                        }
                                    }
                                }
                            }
                            button {
                                r#type: "button",
                                onclick: move |_| cancel(store),
                                class: "rounded border border-rose-200 bg-rose-50 px-3 py-1.5 text-xs font-medium text-rose-700 hover:bg-rose-100 dark:border-rose-900/50 dark:bg-rose-950/40 dark:text-rose-300 dark:hover:bg-rose-900/60 cursor-pointer",
                                "Cancel"
                            }
                        }
                    } else {
                        div { class: "flex flex-wrap items-center justify-center gap-2",
                            button {
                                r#type: "button",
                                disabled: is_busy,
                                onclick: move |_| open_picker(false),
                                class: "rounded bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700 disabled:opacity-40 cursor-pointer border-0",
                                "Browse files"
                            }
                            button {
                                r#type: "button",
                                disabled: is_busy,
                                onclick: move |_| store.paste_open.set(!store.paste_open()),
                                class: "inline-flex items-center gap-1.5 rounded border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700 cursor-pointer",
                                Icon { name: "clipboard-paste", class: "size-3.5" }
                                "Paste logs"
                            }
                        }
                        p { class: "max-w-lg font-mono-data text-[11px] leading-relaxed text-slate-400 dark:text-slate-500",
                            "Example: 2026-07-24T00:00:10: GET /api/health 200 12.5 ms - 42"
                        }
                    }
                }
            }

            if let Some(pending) = pending_drop() {
                PendingDropBanner {
                    pending: pending,
                    loaded_count: loaded_files.len(),
                    on_append: move |files: Vec<LoadedSource>| execute_append(files),
                    on_replace: move |files: Vec<LoadedSource>| execute_replace(files),
                    on_cancel: move |_| pending_drop.set(None),
                }
            }

            if paste_open {
                div { class: "border-t border-slate-200 px-4 py-3 dark:border-slate-800",
                    label {
                        class: "mb-1.5 block text-xs font-medium text-slate-600 dark:text-slate-300",
                        r#for: "paste-logs",
                        "Paste log lines (not persisted; max ~"
                        {format_bytes(PASTE_WARN_BYTES as u64)}
                        ")"
                    }
                    textarea {
                        id: "paste-logs",
                        value: "{paste_text}",
                        oninput: move |e: Event<FormData>| paste_text.set(e.value()),
                        disabled: is_busy,
                        rows: "6",
                        placeholder: "Paste PM2 stdout/stderr here…",
                        class: "w-full resize-y rounded border border-slate-200 bg-slate-50 px-3 py-2 font-mono-data text-xs text-slate-800 placeholder:text-slate-400 focus:border-blue-500 focus:bg-white dark:border-slate-700 dark:bg-slate-950 dark:text-slate-100 dark:placeholder:text-slate-500 dark:focus:bg-slate-900",
                    }
                    div { class: "mt-2 flex justify-end",
                        button {
                            r#type: "button",
                            disabled: is_busy,
                            onclick: analyze_paste,
                            class: "rounded bg-blue-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-blue-700 disabled:opacity-40 cursor-pointer border-0",
                            "Analyze paste"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LoadedFilesDisplay(files: Vec<LoadedSource>) -> Element {
    if files.is_empty() {
        return rsx! {
            span { class: "truncate text-xs font-medium text-slate-700 dark:text-slate-300",
                "Logs loaded"
            }
        };
    }
    let shown: Vec<LoadedSource> = files.iter().take(5).cloned().collect();
    let extra = files.len().saturating_sub(5);
    rsx! {
        div { class: "flex min-w-0 flex-wrap items-center gap-1.5",
            span { class: "text-xs font-semibold text-slate-700 dark:text-slate-300",
                "{files.len()} file"
                if files.len() > 1 { "s" }
                ":"
            }
            for file in shown {
                span {
                    key: "{file.name()}-{file.size()}",
                    class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2 py-0.5 font-mono-data text-[11px] text-slate-700 dark:bg-slate-800 dark:text-slate-300",
                    title: "{file.name()} ({format_bytes(file.size())})",
                    span { class: "max-w-[150px] truncate", "{file.name()}" }
                    span { class: "text-[10px] text-slate-400 dark:text-slate-500",
                        "{format_bytes(file.size())}"
                    }
                }
            }
            if extra > 0 {
                span { class: "text-[11px] font-medium text-slate-500 dark:text-slate-400",
                    "+{extra} more"
                }
            }
        }
    }
}

#[component]
fn PendingDropBanner(
    pending: Vec<LoadedSource>,
    loaded_count: usize,
    on_append: EventHandler<Vec<LoadedSource>>,
    on_replace: EventHandler<Vec<LoadedSource>>,
    on_cancel: EventHandler<()>,
) -> Element {
    let count = pending.len();
    let append_payload = pending.clone();
    let replace_payload = pending.clone();
    rsx! {
        div { class: "border-t border-blue-200 bg-blue-50/90 p-3.5 text-left dark:border-blue-900 dark:bg-slate-800",
            div { class: "mx-auto max-w-md",
                p { class: "text-xs font-semibold text-slate-800 dark:text-slate-200",
                    "You dropped {count} file"
                    if count > 1 { "s" }
                }
                p { class: "mt-0.5 text-[11px] text-slate-600 dark:text-slate-400",
                    "{loaded_count} file"
                    if loaded_count > 1 { "s currently loaded" } else { " currently loaded" }
                    ". How would you like to proceed?"
                }
                div { class: "mt-2.5 flex flex-wrap items-center gap-2",
                    button {
                        r#type: "button",
                        onclick: move |_| on_append.call(append_payload.clone()),
                        class: "inline-flex items-center gap-1 rounded bg-blue-600 px-2.5 py-1 text-xs font-medium text-white hover:bg-blue-700 cursor-pointer border-0",
                        Icon { name: "plus", class: "size-3.5" }
                        "Append to current"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| on_replace.call(replace_payload.clone()),
                        class: "inline-flex items-center gap-1 rounded border border-slate-300 bg-white px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-50 dark:border-slate-600 dark:bg-slate-700 dark:text-slate-200 dark:hover:bg-slate-600 cursor-pointer",
                        Icon { name: "refresh-cw", class: "size-3.5" }
                        "Replace current"
                    }
                    button {
                        r#type: "button",
                        onclick: move |_| on_cancel.call(()),
                        class: "rounded p-1 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 cursor-pointer border-0 bg-transparent",
                        title: "Cancel",
                        Icon { name: "x", class: "size-3.5" }
                    }
                }
            }
        }
    }
}
