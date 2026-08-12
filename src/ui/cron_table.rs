use dioxus::prelude::*;
use crate::core::models::CronJobStats;

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

fn format_ms(val: f32) -> String {
    if val >= 1000.0 {
        format!("{:.2} s", val / 1000.0)
    } else {
        format!("{:.1} ms", val)
    }
}

fn build_cron_tsv(jobs: &[CronJobStats]) -> String {
    let mut tsv = String::from("Job\tRuns\tStarts\tFails\tAvg (ms)\tp95 (ms)\tp99 (ms)\tMax (ms)\tLast (ms)\n");
    for j in jobs {
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\n",
            j.name,
            j.total_runs,
            j.starts,
            j.failed_runs,
            j.avg_duration_ms,
            j.p95_ms,
            j.p99_ms,
            j.max_duration_ms,
            j.last_duration_ms
        ));
    }
    tsv
}

#[component]
pub fn CronTable(
    jobs: Signal<Vec<CronJobStats>>,
    on_toast: EventHandler<String>,
) -> Element {
    let mut cron_query = use_signal(|| String::new());
    let mut cron_min_ms = use_signal(|| 0.0f32);
    let mut cron_failed_only = use_signal(|| false);
    let mut cron_sort_key = use_signal(|| "p95Ms".to_string());

    let filtered_jobs = use_memo(move || {
        let mut list = jobs();
        let q = cron_query().trim().to_lowercase();
        if !q.is_empty() {
            list.retain(|j| j.name.to_lowercase().contains(&q));
        }
        let min_ms = cron_min_ms();
        if min_ms > 0.0 {
            list.retain(|j| j.p95_ms >= min_ms || j.max_duration_ms >= min_ms);
        }
        if cron_failed_only() {
            list.retain(|j| j.failed_runs > 0);
        }

        let sk = cron_sort_key();
        list.sort_by(|a, b| {
            let cmp = match sk.as_str() {
                "runs" => b.total_runs.cmp(&a.total_runs),
                "fails" => b.failed_runs.cmp(&a.failed_runs),
                "avgMs" => b.avg_duration_ms.partial_cmp(&a.avg_duration_ms).unwrap_or(std::cmp::Ordering::Equal),
                "maxMs" => b.max_duration_ms.partial_cmp(&a.max_duration_ms).unwrap_or(std::cmp::Ordering::Equal),
                "p99Ms" => b.p99_ms.partial_cmp(&a.p99_ms).unwrap_or(std::cmp::Ordering::Equal),
                _ => b.p95_ms.partial_cmp(&a.p95_ms).unwrap_or(std::cmp::Ordering::Equal),
            };
            cmp
        });

        list
    });

    let copy_tsv = move |_| {
        let fj = filtered_jobs.read();
        if fj.is_empty() {
            return;
        }
        let tsv = build_cron_tsv(&fj);
        if let Ok(mut ctx) = arboard::Clipboard::new() {
            let _ = ctx.set_text(tsv);
            on_toast.call("Cron table copied — paste into Excel".to_string());
        } else {
            on_toast.call("Copied to clipboard".to_string());
        }
    };

    if jobs().is_empty() {
        return rsx! {};
    }

    let fj_ref = filtered_jobs.read();

    rsx! {
        section { class: "overflow-hidden rounded border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900",
            div { class: "flex flex-wrap items-center justify-between gap-2 border-b border-slate-200 px-3 py-2 dark:border-slate-800",
                h2 { class: "text-xs font-semibold uppercase tracking-wide text-slate-600 dark:text-slate-300",
                    "Cron jobs"
                }
                div { class: "flex flex-wrap items-center gap-2",
                    input {
                        r#type: "search",
                        value: "{cron_query}",
                        oninput: move |e: Event<FormData>| cron_query.set(e.value()),
                        placeholder: "Filter jobs…",
                        class: "rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-800 placeholder:text-slate-400 focus:border-blue-500 dark:border-slate-700 dark:bg-slate-950 dark:text-slate-100 dark:focus:border-blue-400 w-40"
                    }
                    input {
                        r#type: "number",
                        min: "0",
                        value: "{cron_min_ms}",
                        oninput: move |e: Event<FormData>| cron_min_ms.set(e.value().parse::<f32>().unwrap_or(0.0)),
                        placeholder: "Min ms",
                        class: "rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-800 placeholder:text-slate-400 focus:border-blue-500 dark:border-slate-700 dark:bg-slate-950 dark:text-slate-100 dark:focus:border-blue-400 w-20"
                    }
                    label { class: "flex items-center gap-1.5 text-[11px] text-slate-600 dark:text-slate-400 cursor-pointer",
                        input {
                            r#type: "checkbox",
                            checked: "{cron_failed_only}",
                            onchange: move |e: Event<FormData>| cron_failed_only.set(e.value() == "true")
                        }
                        "Failures only"
                    }
                    select {
                        value: "{cron_sort_key}",
                        onchange: move |e: Event<FormData>| cron_sort_key.set(e.value()),
                        class: "rounded border border-slate-200 bg-white px-2 py-1.5 text-xs text-slate-800 focus:border-blue-500 dark:border-slate-700 dark:bg-slate-950 dark:text-slate-100 dark:focus:border-blue-400",
                        option { value: "p95Ms", "p95" }
                        option { value: "p99Ms", "p99" }
                        option { value: "avgMs", "avg" }
                        option { value: "maxMs", "max" }
                        option { value: "runs", "runs" }
                        option { value: "fails", "fails" }
                    }
                    button {
                        r#type: "button",
                        onclick: copy_tsv,
                        disabled: fj_ref.is_empty(),
                        class: "inline-flex items-center gap-1 text-[11px] font-medium text-slate-500 hover:text-slate-800 disabled:opacity-40 dark:text-slate-400 dark:hover:text-slate-200 cursor-pointer bg-transparent border-0",
                        svg { width: "12", height: "12", class: "size-3", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            rect { x: "9", y: "9", width: "13", height: "13", rx: "2", ry: "2" }
                            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
                        }
                        "Copy TSV"
                    }
                }
            }

            if fj_ref.is_empty() {
                div { class: "px-3 py-8 text-center text-sm text-slate-400 dark:text-slate-500",
                    "No cron jobs match filters."
                }
            } else {
                div { class: "max-h-[360px] overflow-auto",
                    div { class: "grid grid-cols-[minmax(0,1.2fr)_56px_56px_56px_64px_64px_64px_64px_64px] items-center gap-1 border-b border-slate-100 bg-slate-50 px-3 py-2 text-[10px] font-semibold uppercase tracking-wide text-slate-500 dark:border-slate-800 dark:bg-slate-950 dark:text-slate-400 sticky top-0 z-10",
                        div { "Job" }
                        div { class: "text-right", "Runs" }
                        div { class: "text-right", "Starts" }
                        div { class: "text-right", "Fails" }
                        div { class: "text-right", "Avg" }
                        div { class: "text-right", "p95" }
                        div { class: "text-right", "p99" }
                        div { class: "text-right", "Max" }
                        div { class: "text-right", "Last" }
                    }
                    for (i, row) in fj_ref.iter().enumerate() {
                        {
                            let is_even = i % 2 == 0;
                            let row_class = if is_even {
                                "grid grid-cols-[minmax(0,1.2fr)_56px_56px_56px_64px_64px_64px_64px_64px] items-center gap-1 border-b border-slate-100 px-3 py-1.5 text-xs transition-colors dark:border-slate-800/60 bg-white dark:bg-slate-900 hover:dark:bg-blue-950/30"
                            } else {
                                "grid grid-cols-[minmax(0,1.2fr)_56px_56px_56px_64px_64px_64px_64px_64px] items-center gap-1 border-b border-slate-100 px-3 py-1.5 text-xs transition-colors dark:border-slate-800/60 bg-slate-50/80 dark:bg-[rgb(11,18,37)] hover:dark:bg-blue-950/30"
                            };
                            let last_dur_str = if row.last_duration_ms > 0.0 { format_ms(row.last_duration_ms) } else { "-".to_string() };
                            rsx! {
                                div { key: "{row.name}", class: "{row_class}",
                                    div { class: "truncate font-mono-data text-[11px] text-slate-800 dark:text-slate-200", title: "{row.name}", "{row.name}" }
                                    div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300 font-mono-data", "{format_num(row.total_runs)}" }
                                    div { class: "text-right tabular-nums text-slate-400 dark:text-slate-500 font-mono-data", "{format_num(row.starts)}" }
                                    div {
                                        class: if row.failed_runs > 0 { "text-right tabular-nums font-bold text-rose-600 dark:text-rose-500 font-mono-data" } else { "text-right tabular-nums text-slate-400 dark:text-slate-600 font-mono-data" },
                                        "{format_num(row.failed_runs)}"
                                    }
                                    div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300 font-mono-data", "{format_ms(row.avg_duration_ms as f32)}" }
                                    div { class: "text-right font-bold tabular-nums text-blue-600 dark:text-blue-400 font-mono-data", "{format_ms(row.p95_ms)}" }
                                    div { class: "text-right tabular-nums text-slate-700 dark:text-slate-300 font-mono-data", "{format_ms(row.p99_ms)}" }
                                    div { class: "text-right font-bold tabular-nums text-amber-700 dark:text-amber-400 font-mono-data", "{format_ms(row.max_duration_ms)}" }
                                    div { class: "text-right tabular-nums text-slate-400 dark:text-slate-500 font-mono-data", "{last_dur_str}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
