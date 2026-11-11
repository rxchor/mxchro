use crate::domain::types::{Collection, Macro};
use std::sync::Arc;

/// Repository errors retain the existing technical messages. They are not UI text.
/// Full typed storage error propagation is a separate, still pending hardening task.
pub trait MacroRepository: Send + Sync {
    fn load_all(&self) -> Vec<Macro>;
    fn insert(&self, value: &Macro) -> Result<(), String>;
    fn delete(&self, key: &str) -> Result<(), String>;
    fn set_enabled(&self, key: &str, enabled: bool) -> Result<(), String>;
    fn replace(&self, old_key: &str, value: &Macro) -> Result<(), String>;
    fn wipe_data(&self) -> Result<(), String>;
    fn set_toggle_hotkey(&self, key: &str, hotkey: Option<String>) -> Result<(), String>;
}

pub trait CollectionRepository: Send + Sync {
    fn load_all(&self) -> Vec<Collection>;
    fn insert(&self, value: &Collection) -> Result<(), String>;
    /// Remove the collection and detach its macros, preserving the macros themselves.
    fn delete(&self, id: &str) -> Result<(), String>;
    /// Persist collection AND all associated macro states in one transaction.
    fn set_enabled_with_macros(&self, id: &str, enabled: bool) -> Result<(), String>;
    fn update_name(&self, id: &str, name: &str) -> Result<(), String>;
    fn reorder(&self, ids: &[String]) -> Result<(), String>;
    fn get_max_order(&self) -> i32;
    fn wipe_data(&self) -> Result<(), String>;
    fn set_toggle_hotkey(&self, id: &str, hotkey: Option<String>) -> Result<(), String>;
}

pub struct Repositories {
    pub macro_repo: Arc<dyn MacroRepository>,
    pub collection_repo: Arc<dyn CollectionRepository>,
}

impl Repositories {
    pub fn wipe_data(&self) -> Result<(), String> {
        self.macro_repo.wipe_data()?;
        self.collection_repo.wipe_data()
    }
}
