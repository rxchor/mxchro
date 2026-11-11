use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::application::{
    dto::CollectionDTO,
    ports::{InputBackend, Repositories},
};
use crate::domain::types::Collection;

use super::hotkeys;
use super::macros::{MacroInstance, ToggleTarget};
use crate::application::error::CommandError;

pub struct CollectionManager {
    pub(super) collections: Arc<RwLock<HashMap<String, Collection>>>,
    macros: Arc<RwLock<HashMap<u16, MacroInstance>>>,
    toggle_hotkeys: Arc<RwLock<HashMap<u16, ToggleTarget>>>,
    key_mapper: Arc<dyn InputBackend>,
    repositories: Arc<Repositories>,
}

impl CollectionManager {
    pub fn new(
        collections: Arc<RwLock<HashMap<String, Collection>>>,
        macros: Arc<RwLock<HashMap<u16, MacroInstance>>>,
        toggle_hotkeys: Arc<RwLock<HashMap<u16, ToggleTarget>>>,
        key_mapper: Arc<dyn InputBackend>,
        repositories: Arc<Repositories>,
    ) -> Self {
        let manager = CollectionManager {
            collections,
            macros,
            toggle_hotkeys,
            key_mapper,
            repositories,
        };

        manager.load_collections();

        manager
    }

    pub fn create_collection(&self, name: String) -> Option<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let order = self.repositories.collection_repo.get_max_order() + 1;
        let collection = Collection {
            id: id.clone(),
            name,
            enabled: true,
            order,
            toggle_enabled_hotkey: None,
        };

        if let Err(e) = self.repositories.collection_repo.insert(&collection) {
            log::error!("Failed to persist new collection: {}", e);
            return None;
        }

        let mut collections = self.collections.write().unwrap();
        collections.insert(id.clone(), collection);
        log::info!("Collection created: {}", id);
        Some(id)
    }

    pub fn delete_collection(&self, collection_id: &str) -> bool {
        let mut collections = self.collections.write().unwrap();
        if !collections.contains_key(collection_id) {
            log::warn!(
                "delete_collection: collection '{}' not found",
                collection_id
            );
            return false;
        }

        if let Err(e) = self.repositories.collection_repo.delete(collection_id) {
            log::error!("Failed to persist collection deletion: {}", e);
            return false;
        }
        collections.remove(collection_id);

        // Detach macros from this collection.
        let mut macros = self.macros.write().unwrap();
        for instance in macros.values_mut() {
            if instance.collection_id.as_deref() == Some(collection_id) {
                instance.collection_id = None;
            }
        }
        hotkeys::refresh(
            &self.toggle_hotkeys,
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        );

        log::info!("Collection deleted: {}", collection_id);
        true
    }

    pub fn enable_collection(&self, collection_id: &str) -> bool {
        let mut collections = self.collections.write().unwrap();
        collections
            .get_mut(collection_id)
            .is_some_and(|collection| self.set_collection_enabled(collection, true))
    }

    pub fn disable_collection(&self, collection_id: &str) -> bool {
        let mut collections = self.collections.write().unwrap();
        collections
            .get_mut(collection_id)
            .is_some_and(|collection| self.set_collection_enabled(collection, false))
    }

    pub fn list_collections(&self) -> Vec<CollectionDTO> {
        let collections = self.collections.read().unwrap();
        let macros = self.macros.read().unwrap();

        let mut views: Vec<CollectionDTO> = collections
            .values()
            .map(|c| {
                let macros_count = macros
                    .values()
                    .filter(|m| m.collection_id.as_deref() == Some(&c.id))
                    .count();
                CollectionDTO {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    enabled: c.enabled,
                    macros_count,
                    order: c.order,
                    toggle_enabled_hotkey: c.toggle_enabled_hotkey.clone(),
                }
            })
            .collect();

        views.sort_by(|a, b| a.order.cmp(&b.order));
        views
    }

    pub fn rename_collection(&self, collection_id: &str, name: String) -> bool {
        let mut collections = self.collections.write().unwrap();
        if let Some(collection) = collections.get_mut(collection_id) {
            let old_name = collection.name.clone();
            collection.name = name.clone();

            if let Err(e) = self
                .repositories
                .collection_repo
                .update_name(collection_id, &name)
            {
                log::error!("Failed to persist collection rename: {}", e);
                collection.name = old_name;
                return false;
            }

            log::info!("Collection '{}' renamed to '{}'", collection_id, name);
            true
        } else {
            log::warn!(
                "rename_collection: collection '{}' not found",
                collection_id
            );
            false
        }
    }

    pub fn reorder_collections(&self, ordered_ids: Vec<String>) -> bool {
        let collections = self.collections.read().unwrap();
        for id in &ordered_ids {
            if !collections.contains_key(id) {
                log::warn!("reorder_collections: collection '{}' not found", id);
                return false;
            }
        }
        drop(collections);

        if let Err(e) = self.repositories.collection_repo.reorder(&ordered_ids) {
            log::error!("Failed to persist collection reorder: {}", e);
            return false;
        }

        let mut collections = self.collections.write().unwrap();
        for (index, id) in ordered_ids.iter().enumerate() {
            if let Some(collection) = collections.get_mut(id) {
                collection.order = index as i32;
            }
        }

        log::info!("Collections reordered");
        true
    }

    pub fn set_collection_toggle_hotkey(
        &self,
        collection_id: &str,
        hotkey: Option<String>,
    ) -> Result<(), CommandError> {
        let mut collections = self.collections.write().unwrap();
        if !collections.contains_key(collection_id) {
            return Err(CommandError::CollectionNotFound);
        }
        let macros = self.macros.read().unwrap();
        hotkeys::validate(
            hotkey.as_deref(),
            false,
            Some(&ToggleTarget::Collection(collection_id.into())),
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        )?;
        if let Some(collection) = collections.get_mut(collection_id) {
            let hotkey_str = hotkey.clone();
            if let Err(e) = self
                .repositories
                .collection_repo
                .set_toggle_hotkey(collection_id, hotkey.clone())
            {
                log::error!(
                    "Failed to persist toggle hotkey for collection '{}': {}",
                    collection_id,
                    e
                );
                return Err(CommandError::OperationFailed);
            }
            collection.toggle_enabled_hotkey = hotkey;
            hotkeys::refresh(
                &self.toggle_hotkeys,
                &collections,
                &macros,
                self.key_mapper.as_ref(),
            );

            log::info!(
                "Collection '{}' toggle hotkey set to {:?}",
                collection_id,
                hotkey_str
            );
            Ok(())
        } else {
            log::warn!(
                "set_collection_toggle_hotkey: collection '{}' not found",
                collection_id
            );
            Err(CommandError::CollectionNotFound)
        }
    }

    /// Toggle a collection and all its macros (called from the toggle hotkey).
    /// Return the new state on success, or None if missing or persistence fails.
    pub fn toggle_collection_enabled(&self, collection_id: &str) -> Option<bool> {
        let mut collections = self.collections.write().unwrap();
        let collection = collections.get_mut(collection_id)?;
        let enabled = !collection.enabled;
        self.set_collection_enabled(collection, enabled)
            .then_some(enabled)
    }

    // The caller holds the collections lock. Always acquire collections before
    // macros, including list/delete operations, to avoid lock-order inversion.
    fn set_collection_enabled(&self, collection: &mut Collection, enabled: bool) -> bool {
        let mut macros = self.macros.write().unwrap();
        if let Err(e) = self
            .repositories
            .collection_repo
            .set_enabled_with_macros(&collection.id, enabled)
        {
            log::error!("Failed to persist collection state: {}", e);
            return false;
        }

        for instance in macros.values_mut() {
            if instance.collection_id.as_deref() == Some(&collection.id) {
                if enabled {
                    instance.enable();
                } else {
                    instance.disable();
                }
            }
        }
        collection.enabled = enabled;
        log::info!("Collection '{}' enabled={}", collection.id, enabled);
        true
    }

    fn load_collections(&self) {
        let collections = self.repositories.collection_repo.load_all();
        let mut coll_map = self.collections.write().unwrap();

        for collection in collections {
            coll_map.insert(collection.id.clone(), collection);
        }
        log::info!("Loaded {} collections from persistence", coll_map.len());
        let macros = self.macros.read().unwrap();
        hotkeys::refresh(
            &self.toggle_hotkeys,
            &coll_map,
            &macros,
            self.key_mapper.as_ref(),
        );
    }

    pub fn clear_collections(&self) {
        let mut collections = self.collections.write().unwrap();
        collections.clear();
        let macros = self.macros.read().unwrap();
        hotkeys::refresh(
            &self.toggle_hotkeys,
            &collections,
            &macros,
            self.key_mapper.as_ref(),
        );
    }
}
