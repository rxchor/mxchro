use crate::core::macro_manager::MacroManager;
use crate::core::types::{Macro, MacroView};

pub struct Commands {
    manager: &'static MacroManager,
}

impl Commands {
    pub fn new(manager: &'static MacroManager) -> Self {
        Commands { manager }
    }

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

    pub fn wipe_data(&self) -> Result<(), String> {
        self.manager.wipe_data()
    }
}
