use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    core::{
        managers::{
            collection_manager::CollectionManager,
            macro_manager::{MacroInstance, MacroManager, ToggleTarget},
        },
        persistence::Persistence,
        types::Collection,
    },
    infrastructure::input::mapper::KeyMapper,
};

pub mod collection_manager;
pub mod macro_manager;

pub struct Managers {
    persistence: Arc<Persistence>,
    pub collection_manager: Arc<CollectionManager>,
    pub macro_manager: Arc<MacroManager>,
}

impl Managers {
    pub fn new() -> Self {
        let collections: Arc<RwLock<HashMap<String, Collection>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let macros: Arc<RwLock<HashMap<u16, MacroInstance>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let toggle_hotkeys: Arc<RwLock<HashMap<u16, ToggleTarget>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let key_mapper: Arc<KeyMapper> = Arc::new(KeyMapper::new());
        let persistence: Arc<Persistence> = Arc::new(Persistence::new());

        let collection_manager: Arc<CollectionManager> = Arc::new(CollectionManager::new(
            Arc::clone(&collections),
            Arc::clone(&macros),
            Arc::clone(&toggle_hotkeys),
            Arc::clone(&key_mapper),
            Arc::clone(&persistence),
        ));

        let macro_manager: Arc<MacroManager> = Arc::new(MacroManager::new(
            Arc::clone(&macros),
            Arc::clone(&collections),
            Arc::clone(&toggle_hotkeys),
            Arc::clone(&key_mapper),
            Arc::clone(&persistence),
        ));

        Managers {
            persistence,
            collection_manager,
            macro_manager,
        }
    }

    pub fn wipe_data(&self) -> Result<(), String> {
        // Limpiar macros
        self.macro_manager.clear_macros();

        // Limpiar colecciones
        self.collection_manager.clear_collections();

        // Limpiar la base de datos
        self.persistence.wipe_data()?;

        log::info!("Wiped all macro and collection data");
        Ok(())
    }
}
