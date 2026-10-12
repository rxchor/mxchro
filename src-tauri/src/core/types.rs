use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════
// ENTITIES (core/types.rs) - Dominio puro, sin dependencias externas
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Toggle,
    Hold,
    Once,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "key")]
    Key { key: String, press: bool },
    #[serde(rename = "wait")]
    Wait { duration_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub trigger_key: String,
    pub mode: Mode,
    pub actions: Vec<Action>,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
    pub collection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}

// ═══════════════════════════════════════════════════════════════
// VIEWS - Proyecciones para UI
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroView {
    pub trigger_key: String,
    pub mode: Mode,
    pub actions_count: usize,
    pub block_original_event: bool,
    pub ignore_injected_keys: bool,
    pub enabled: bool,
    pub collection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionView {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub macros_count: usize,
}

pub type ActionCallback = Arc<dyn Fn() + Send + Sync + 'static>;
