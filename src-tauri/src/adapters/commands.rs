use crate::core::macro_manager::MacroManager;
use crate::core::types::{MacroConfig, MacroSummary};

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

    pub fn add_macro(&self, config: MacroConfig) -> bool {
        self.manager.add_macro(config)
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

    pub fn list_macros(&self) -> Vec<MacroSummary> {
        self.manager.list_macros()
    }
}
