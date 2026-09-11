//! App mode store — mirrors `src/store/appModeStore.ts`.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::utils::persist;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Pm2,
    Mongo,
}

#[derive(Serialize, Deserialize)]
struct PersistedMode {
    state: ModeState,
    version: u32,
}

#[derive(Serialize, Deserialize)]
struct ModeState {
    mode: AppMode,
}

#[derive(Clone, Copy)]
pub struct AppModeStore {
    pub mode: Signal<AppMode>,
}

impl AppModeStore {
    pub fn mode(&self) -> AppMode {
        (self.mode)()
    }

    pub fn new() -> Self {
        Self {
            mode: Signal::new(AppMode::Pm2),
        }
    }
}

impl Default for AppModeStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_app_mode_store() -> AppModeStore {
    use_context::<AppModeStore>()
}

pub fn set_mode(mut store: AppModeStore, mode: AppMode) {
    store.mode.set(mode);
    let envelope = PersistedMode {
        state: ModeState { mode },
        version: 0,
    };
    let json = serde_json::to_string(&envelope).unwrap_or_default();
    persist::set_item("app-analyzer-mode", &json);
}

pub fn toggle_mode(store: AppModeStore) {
    let next = if store.mode() == AppMode::Pm2 {
        AppMode::Mongo
    } else {
        AppMode::Pm2
    };
    set_mode(store, next);
}

pub async fn restore_persisted_mode(mut store: AppModeStore) {
    if let Some(raw) = persist::load_item("app-analyzer-mode").await {
        if let Ok(envelope) = serde_json::from_str::<PersistedMode>(&raw) {
            store.mode.set(envelope.state.mode);
        }
    }
}
