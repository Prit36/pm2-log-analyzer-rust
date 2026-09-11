//! `SkippedDisclosure` — port of `src/components/SkippedDisclosure.tsx`.

use dioxus::prelude::*;

use crate::store::use_analysis_store;
use crate::utils::format::format_num;

#[component]
pub fn SkippedDisclosure() -> Element {
    let store = use_analysis_store();
    if !store.has_data() {
        return rsx! {};
    }
    let Some(result) = store.result() else {
        return rsx! {};
    };
    let unmatched_count = result.unmatched_count;
    if unmatched_count == 0 {
        return rsx! {};
    }
    let sample = result.unmatched_sample;
    rsx! {
        details { class: "rounded border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900",
            summary { class: "cursor-pointer px-3 py-2 text-xs font-medium text-slate-600 hover:bg-slate-50 dark:text-slate-300 dark:hover:bg-slate-800",
                "{format_num(unmatched_count)} lines skipped"
                span { class: "ml-2 font-normal text-slate-400 dark:text-slate-500",
                    "(non-HTTP / unmatched)"
                }
            }
            div { class: "border-t border-slate-100 px-3 py-2 dark:border-slate-800",
                ul { class: "max-h-48 space-y-1 overflow-auto font-mono-data text-[11px] text-slate-600 dark:text-slate-400",
                    for (i, line) in sample.iter().enumerate() {
                        li { key: "{i}", class: "truncate", title: "{line}", "{line}" }
                    }
                }
                if unmatched_count > sample.len() as u64 {
                    p { class: "mt-2 text-[11px] text-slate-400 dark:text-slate-500",
                        "Showing {sample.len()} of {format_num(unmatched_count)} samples"
                    }
                }
            }
        }
    }
}
