use serde::{Deserialize, Serialize};

use super::key_mode::Mode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEventDto {
    pub key: Option<String>,
    pub sleep: Option<u64>,
    pub hold: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDto {
    pub trigger_key: String,
    pub mode: Mode,
    pub actions: Vec<KeyEventDto>,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroSummaryDto {
    pub trigger_key: String,
    pub mode: Mode,
    pub actions_count: usize,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
}
