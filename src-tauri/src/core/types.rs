use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Toggle,
    Hold,
    Once,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionConfig {
    #[serde(rename = "key")]
    Key { key: String, press: bool },
    #[serde(rename = "wait")]
    Wait { duration_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroConfig {
    pub trigger_key: String,
    pub mode: Mode,
    pub actions: Vec<ActionConfig>,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroSummary {
    pub trigger_key: String,
    pub mode: Mode,
    pub actions_count: usize,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
}

pub type Action = Arc<dyn Fn() + Send + Sync + 'static>;
