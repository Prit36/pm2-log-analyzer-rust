use dioxus::prelude::*;
use crate::core::models::EndpointStats;

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

fn format_ms(val: f64) -> String {
    if val >= 1000.0 {
        format!("{:.2} s", val / 1000.0)
    } else {
        format!("{:.1} ms", val)
    }
}

fn build_api_tsv(endpoints: &[EndpointStats]) -> String {
    let mut tsv = String::from("Method\tEndpoint\tCount\tAvg (ms)\tp95 (ms)\tp99 (ms)\tMax (ms)\tErrors\n");
    for r in endpoints {
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{}\n",
            r.method.as_str(),
            r.path,
            r.total_calls,
            r.avg_duration_ms(),
            r.p95_ms,
            r.p99_ms,
            r.max_duration_ms,
            r.failed_calls
        ));
    }
    tsv
}

#[component]
pub fn ApiTable(
    endpoints: Signal<Vec<EndpointStats>>,
    on_toast: EventHandler<String>,
) -> Element {
    let copy_tsv = move |_| {
        let ep = endpoints();
        if ep.is_empty() {
            return;
        }
        let tsv = build_api_tsv(&ep);
        if let Ok(mut ctx) = arboard::Clipboard::new() {
            let _ = ctx.set_text(tsv);
            on_toast.call("API table copied — paste into Excel".to_string());
        } else {
            on_toast.call("Copied to clipboard".to_string());
        }
    };

    rsx! {
        section { class: "overflow-hidden rounded border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900",
            div { class: "flex items-center justify-between border-b border-slate-200 px-3 py-2 dark:border-slate-800",
                h2 { class: "text-xs font-semibold uppercase tracking-wide text-slate-600 dark:text-slate-300",
                    "Slow API endpoints"
                }
                button {
                    r#type: "button",
                    onclick: copy_tsv,
                    disabled: endpoints().is_empty(),
                    class: "inline-flex items-center gap-1 text-[11px] font-medium text-slate-500 hover:text-slate-800 disabled:opacity-40 dark:text-slate-400 dark:hover:text-slate-200 cursor-pointer bg-transparent border-0",
                    svg { width: "12", height: "12", class: "size-3", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                        rect { x: "9", y: "9", width: "13", height: "13", rx: "2", ry: "2" }
                        path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
                    }
                    "Copy TSV"
                }
            }

            if endpoints().is_empty() {
                div { class: "px-3 py-10 text-center text-sm text-slate-400 dark:text-slate-500",
                    "Upload or paste logs to see endpoints here."
                }
            } else {
                div { class: "max-h-[420px] overflow-auto",
                    div { class: "grid grid-cols-[minmax(0,1fr)_64px_64px_64px_64px_64px_56px] items-center gap-1 border-b border-slate-100 bg-slate-50 px-3 py-2 text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:border-slate-800 dark:bg-slate-950 dark:text-slate-400 sticky top-0 z-10",
                        div { "Endpoint" }
                        div { class: "text-right", "Count" }
                        div { class: "text-right", "Avg" }
                        div { class: "text-right", "p95" }
                        div { class: "text-right", "p99" }
                        div { class: "text-right", "Max" }
                        div { class: "text-right", "Err" }
                    }
                    for (i, row) in endpoints().iter().enumerate() {
                        {
                            let method_str = row.method.as_str();
                            let badge_class = match method_str {
                                "GET" => "inline-block shrink-0 rounded px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wider ring-1 dark:ring-0 bg-sky-50 text-sky-700 ring-sky-200 dark:border dark:border-sky-600/60 dark:bg-[#062238] dark:text-[#38bdf8]",
                                "POST" => "inline-block shrink-0 rounded px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wider ring-1 dark:ring-0 bg-emerald-50 text-emerald-700 ring-emerald-200 dark:border dark:border-emerald-600/60 dark:bg-[#06261c] dark:text-[#34d399]",
                                "PUT" | "PATCH" => "inline-block shrink-0 rounded px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wider ring-1 dark:ring-0 bg-amber-50 text-amber-700 ring-amber-200 dark:border dark:border-amber-600/60 dark:bg-[#381a06] dark:text-[#fbbf24]",
                                "DELETE" => "inline-block shrink-0 rounded px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wider ring-1 dark:ring-0 bg-rose-50 text-rose-700 ring-rose-200 dark:border dark:border-rose-600/60 dark:bg-[#3d0818] dark:text-[#fb7185]",
                                _ => "inline-block shrink-0 rounded px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wider ring-1 dark:ring-0 bg-slate-50 text-slate-600 ring-slate-200 dark:border dark:border-slate-700/60 dark:bg-slate-900 dark:text-slate-400",
                            };
                            let is_even = i % 2 == 0;
                            let row_class = if is_even {
                                "grid grid-cols-[minmax(0,1fr)_64px_64px_64px_64px_64px_56px] items-center gap-1 border-b border-slate-100 px-3 py-1.5 text-xs transition-colors dark:border-slate-800/60 bg-white dark:bg-slate-900 hover:dark:bg-blue-950/30"
                            } else {
                                "grid grid-cols-[minmax(0,1fr)_64px_64px_64px_64px_64px_56px] items-center gap-1 border-b border-slate-100 px-3 py-1.5 text-xs transition-colors dark:border-slate-800/60 bg-slate-50/80 dark:bg-[rgb(11,18,37)] hover:dark:bg-blue-950/30"
                            };
                            let path = row.path.clone();

                            rsx! {
                                div { key: "{row.path}_{method_str}", class: "{row_class}",
                                    div { class: "flex min-w-0 items-center gap-2",
                                        span { class: "{badge_class}", "{method_str}" }
                                        button {
                                            r#type: "button",
                                            title: "Copy path",
                                            onclick: move |_| {
                                                if let Ok(mut ctx) = arboard::Clipboard::new() {
                                                    let _ = ctx.set_text(path.clone());
                                                }
                                                on_toast.call("Path copied".to_string());
                                            },
                                            class: "truncate text-left font-mono-data text-[11px] text-slate-800 hover:text-blue-700 dark:text-slate-200 dark:hover:text-blue-400 bg-transparent border-0 cursor-pointer p-0",
                                            "{row.path}"
                                        }
                                    }
                                    div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300 font-mono-data", "{format_num(row.total_calls)}" }
                                    div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300 font-mono-data", "{format_ms(row.avg_duration_ms())}" }
                                    div { class: "text-right font-bold tabular-nums text-blue-600 dark:text-blue-400 font-mono-data", "{format_ms(row.p95_ms as f64)}" }
                                    div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300 font-mono-data", "{format_ms(row.p99_ms as f64)}" }
                                    div { class: "text-right font-bold tabular-nums text-amber-700 dark:text-amber-400 font-mono-data", "{format_ms(row.max_duration_ms as f64)}" }
                                    div {
                                        class: if row.failed_calls > 0 { "text-right tabular-nums font-bold text-rose-600 dark:text-rose-500 font-mono-data" } else { "text-right tabular-nums text-slate-400 dark:text-slate-600 font-mono-data" },
                                        "{format_num(row.failed_calls)}"
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
