//! MongoDB analyzer UI — full port lands in the Mongo stage.

use dioxus::prelude::*;

#[component]
pub fn MongoHeaderInfo() -> Element {
    rsx! {
        div {
            div { class: "flex items-center gap-2",
                span { class: "text-base font-semibold tracking-tight text-slate-900 dark:text-slate-100",
                    "MongoDB Log Analyzer"
                }
            }
            p { class: "mt-0.5 text-xs text-slate-500 dark:text-slate-400",
                "Slow queries, patterns & user activity"
            }
        }
    }
}

#[component]
pub fn MongoAppView() -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            section { class: "rounded border border-slate-200 bg-white px-6 py-10 text-center text-sm text-slate-500 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-400",
                "MongoDB analyzer is not wired yet."
            }
        }
    }
}
