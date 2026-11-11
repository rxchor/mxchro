use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    application::ports::{InputBackend, Repositories},
    domain::types::Collection,
};
use collections::CollectionManager;
use macros::{MacroInstance, MacroManager, ToggleTarget};

pub mod collections;
mod hotkeys;
pub mod macros;

#[cfg(test)]
#[path = "../../../tests/unit/application/engine.rs"]
mod tests;

pub struct Managers {
    repositories: Arc<Repositories>,
    pub collection_manager: Arc<CollectionManager>,
    pub macro_manager: Arc<MacroManager>,
}

impl Managers {
    pub fn new(key_mapper: Arc<dyn InputBackend>, repositories: Arc<Repositories>) -> Self {
        let collections: Arc<RwLock<HashMap<String, Collection>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let macros: Arc<RwLock<HashMap<u16, MacroInstance>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let toggle_hotkeys: Arc<RwLock<HashMap<u16, ToggleTarget>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let collection_manager: Arc<CollectionManager> = Arc::new(CollectionManager::new(
            Arc::clone(&collections),
            Arc::clone(&macros),
            Arc::clone(&toggle_hotkeys),
            Arc::clone(&key_mapper),
            Arc::clone(&repositories),
        ));

        let macro_manager: Arc<MacroManager> = Arc::new(MacroManager::new(
            Arc::clone(&macros),
            Arc::clone(&collection_manager),
            Arc::clone(&toggle_hotkeys),
            Arc::clone(&key_mapper),
            Arc::clone(&repositories),
        ));

        Managers {
            repositories,
            collection_manager,
            macro_manager,
        }
    }

    pub fn wipe_data(&self) -> Result<(), String> {
        // Clear macros.
        self.macro_manager.clear_macros();

        // Clear collections.
        self.collection_manager.clear_collections();

        // Clear the database.
        self.repositories.wipe_data()?;

        log::info!("Wiped all macro and collection data");
        Ok(())
    }
}
