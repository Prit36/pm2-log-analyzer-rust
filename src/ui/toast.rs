use dioxus::prelude::*;

#[component]
pub fn Toast(
    toast_msg: Signal<Option<String>>,
) -> Element {
    if let Some(msg) = toast_msg() {
        rsx! {
            div { class: "fixed bottom-4 right-4 z-50 rounded bg-slate-900 px-3 py-2 text-xs text-white shadow-lg dark:bg-slate-100 dark:text-slate-900 font-medium",
                "{msg}"
            }
        }
    } else {
        rsx! {}
    }
}
