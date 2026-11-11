use crate::domain::types::{Action, Macro, Mode};

/// Lightweight list projection: a card never needs to clone an action sequence.
#[derive(Debug, Clone)]
pub struct MacroDTO {
    pub trigger_key: String,
    pub mode: Mode,
    pub actions_count: usize,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
    pub collection_id: Option<String>,
    pub toggle_enabled_hotkey: Option<String>,
}

/// Detailed query result requested only when opening the editor.
#[derive(Debug, Clone)]
pub struct MacroEditDTO {
    pub trigger_key: String,
    pub mode: Mode,
    pub actions: Vec<Action>,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
    pub collection_id: Option<String>,
    pub toggle_enabled_hotkey: Option<String>,
}

/// Input to the save use case. It is validated before mutating engine state.
#[derive(Debug, Clone)]
pub struct SaveMacroDTO {
    pub trigger_key: String,
    pub mode: Mode,
    pub actions: Vec<Action>,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
    pub collection_id: Option<String>,
    pub toggle_enabled_hotkey: Option<String>,
}

impl From<&Macro> for MacroDTO {
    fn from(value: &Macro) -> Self {
        Self {
            trigger_key: value.trigger_key.clone(),
            mode: value.mode.clone(),
            actions_count: value.actions.len(),
            block_original_event: value.block_original_event,
            ignore_injected_keys: value.ignore_injected_keys,
            enabled: value.enabled,
            collection_id: value.collection_id.clone(),
            toggle_enabled_hotkey: value.toggle_enabled_hotkey.clone(),
        }
    }
}

impl From<Macro> for MacroEditDTO {
    fn from(value: Macro) -> Self {
        Self {
            trigger_key: value.trigger_key,
            mode: value.mode,
            actions: value.actions,
            block_original_event: value.block_original_event,
            ignore_injected_keys: value.ignore_injected_keys,
            enabled: value.enabled,
            collection_id: value.collection_id,
            toggle_enabled_hotkey: value.toggle_enabled_hotkey,
        }
    }
}

impl From<MacroEditDTO> for SaveMacroDTO {
    fn from(value: MacroEditDTO) -> Self {
        Self {
            trigger_key: value.trigger_key,
            mode: value.mode,
            actions: value.actions,
            block_original_event: value.block_original_event,
            ignore_injected_keys: value.ignore_injected_keys,
            enabled: value.enabled,
            collection_id: value.collection_id,
            toggle_enabled_hotkey: value.toggle_enabled_hotkey,
        }
    }
}

impl From<SaveMacroDTO> for Macro {
    fn from(value: SaveMacroDTO) -> Self {
        Self {
            trigger_key: value.trigger_key,
            mode: value.mode,
            actions: value.actions,
            block_original_event: value.block_original_event,
            ignore_injected_keys: value.ignore_injected_keys,
            enabled: value.enabled,
            collection_id: value.collection_id,
            toggle_enabled_hotkey: value.toggle_enabled_hotkey,
        }
    }
}
