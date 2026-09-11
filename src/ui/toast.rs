//! `Toast` — port of `src/components/Toast.tsx`.

use dioxus::prelude::*;

use crate::store::{use_analysis_store, use_app_mode_store, AppMode};

#[component]
pub fn Toast() -> Element {
    let store = use_analysis_store();
    let app = use_app_mode_store();
    let pm2_toast = store.toast();
    let mongo_toast: Option<String> = None;
    let toast = if app.mode() == AppMode::Mongo {
        mongo_toast.or(pm2_toast)
    } else {
        pm2_toast.or(mongo_toast)
    };
    let Some(toast) = toast else {
        return rsx! {};
    };
    rsx! {
        div {
            role: "status",
            class: "fixed bottom-4 right-4 z-50 max-w-sm rounded-lg border border-slate-200 bg-slate-900 px-3.5 py-2.5 text-xs font-medium text-white shadow-xl dark:border-slate-700 dark:bg-slate-800 dark:text-slate-100",
            "{toast}"
        }
    }
}
