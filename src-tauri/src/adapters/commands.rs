use crate::core::macro_manager::MacroManager;
use crate::core::types::{CollectionView, Macro, MacroView};

pub struct Commands {
    manager: &'static MacroManager,
}

impl Commands {
    pub fn new(manager: &'static MacroManager) -> Self {
        Commands { manager }
    }

    // ── Macros ──────────────────────────────────────────────

    pub fn get_available_keys(&self) -> Vec<String> {
        self.manager.get_available_keys()
    }

    pub fn add_macro(&self, macro_entity: Macro) -> bool {
        self.manager.add_macro(macro_entity)
    }

    pub fn get_macro(&self, trigger_key: String) -> Option<Macro> {
        self.manager.get_macro(&trigger_key)
    }

    pub fn update_macro(&self, old_trigger_key: String, macro_entity: Macro) -> bool {
        self.manager.update_macro(&old_trigger_key, macro_entity)
    }

    pub fn delete_macro(&self, trigger_key: String) -> bool {
        self.manager.delete_macro(&trigger_key)
    }

    pub fn enable_macro(&self, trigger_key: String) -> bool {
        self.manager.enable_macro(&trigger_key)
    }

    pub fn disable_macro(&self, trigger_key: String) -> bool {
        self.manager.disable_macro(&trigger_key)
    }

    pub fn list_macros(&self) -> Vec<MacroView> {
        self.manager.list_macros()
    }

    pub fn move_macro_to_collection(&self, trigger_key: String, collection_id: Option<String>) -> bool {
        self.manager.move_macro_to_collection(&trigger_key, collection_id)
    }

    // ── Collections ─────────────────────────────────────────

    pub fn create_collection(&self, name: String) -> Option<String> {
        self.manager.create_collection(name)
    }

    pub fn delete_collection(&self, collection_id: String) -> bool {
        self.manager.delete_collection(&collection_id)
    }

    pub fn enable_collection(&self, collection_id: String) -> bool {
        self.manager.enable_collection(&collection_id)
    }

    pub fn disable_collection(&self, collection_id: String) -> bool {
        self.manager.disable_collection(&collection_id)
    }

    pub fn list_collections(&self) -> Vec<CollectionView> {
        self.manager.list_collections()
    }

    pub fn rename_collection(&self, collection_id: String, name: String) -> bool {
        self.manager.rename_collection(&collection_id, name)
    }

    pub fn reorder_collections(&self, ordered_ids: Vec<String>) -> bool {
        self.manager.reorder_collections(ordered_ids)
    }

    // ── Utility ─────────────────────────────────────────────

    pub fn wipe_data(&self) -> Result<(), String> {
        self.manager.wipe_data()
    }
}
