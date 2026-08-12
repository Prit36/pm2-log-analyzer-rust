use dioxus::prelude::*;

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

#[component]
pub fn SkippedDisclosure(
    unmatched_count: Signal<u64>,
    samples: Signal<Vec<String>>,
    has_data: Signal<bool>,
) -> Element {
    if !has_data() || unmatched_count() == 0 {
        return rsx! {};
    }

    let count = unmatched_count();
    let sample_list = samples();
    let sample_len = sample_list.len();

    rsx! {
        details { class: "rounded border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900",
            summary { class: "cursor-pointer px-3 py-2 text-xs font-medium text-slate-600 hover:bg-slate-50 dark:text-slate-300 dark:hover:bg-slate-800",
                "{format_num(count)} lines skipped"
                span { class: "ml-2 font-normal text-slate-400 dark:text-slate-500",
                    "(non-HTTP / unmatched)"
                }
            }
            div { class: "border-t border-slate-100 px-3 py-2 dark:border-slate-800",
                ul { class: "max-h-48 space-y-1 overflow-auto font-mono-data text-[11px] text-slate-600 dark:text-slate-400",
                    for line in sample_list.iter() {
                        li { class: "truncate", title: "{line}", "{line}" }
                    }
                }
                if count > sample_len as u64 {
                    p { class: "mt-2 text-[11px] text-slate-400 dark:text-slate-500",
                        "Showing {sample_len} of {format_num(count)} samples"
                    }
                }
            }
        }
    }
}
