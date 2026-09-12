//! App mode state — mirrors `src/store/appModeStore.ts`.

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

pub fn persist_mode(mode: AppMode) {
    let envelope = PersistedMode {
        state: ModeState { mode },
        version: 0,
    };
    let json = serde_json::to_string(&envelope).unwrap_or_default();
    persist::set_item("app-analyzer-mode", &json);
}

pub fn restore_mode() -> Option<AppMode> {
    let raw = persist::load_item("app-analyzer-mode")?;
    serde_json::from_str::<PersistedMode>(&raw)
        .ok()
        .map(|envelope| envelope.state.mode)
}
