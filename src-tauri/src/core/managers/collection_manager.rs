use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::core::persistence::Persistence;
use crate::core::types::{Collection, CollectionView};
use crate::infrastructure::input::mapper::KeyMapper;

use super::macro_manager::{MacroInstance, ToggleTarget};

pub struct CollectionManager {
    collections: Arc<RwLock<HashMap<String, Collection>>>,
    macros: Arc<RwLock<HashMap<u16, MacroInstance>>>,
    toggle_hotkeys: Arc<RwLock<HashMap<u16, ToggleTarget>>>,
    key_mapper: Arc<KeyMapper>,
    persistence: Arc<Persistence>,
}

impl CollectionManager {
    pub fn new(
        collections: Arc<RwLock<HashMap<String, Collection>>>,
        macros: Arc<RwLock<HashMap<u16, MacroInstance>>>,
        toggle_hotkeys: Arc<RwLock<HashMap<u16, ToggleTarget>>>,
        key_mapper: Arc<KeyMapper>,
        persistence: Arc<Persistence>,
    ) -> Self {
        let manager = CollectionManager {
            collections,
            macros,
            toggle_hotkeys,
            key_mapper,
            persistence,
        };

        manager.load_collections();

        manager
    }

    pub fn create_collection(&self, name: String) -> Option<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let order = self.persistence.collection_repo.get_max_order() + 1;
        let collection = Collection {
            id: id.clone(),
            name,
            enabled: true,
            order,
            toggle_enabled_hotkey: None,
        };

        if let Err(e) = self.persistence.collection_repo.insert(&collection) {
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
        if collections.remove(collection_id).is_none() {
            log::warn!(
                "delete_collection: collection '{}' not found",
                collection_id
            );
            return false;
        }

        if let Err(e) = self.persistence.collection_repo.delete(collection_id) {
            log::error!("Failed to persist collection deletion: {}", e);
            return false;
        }

        // Desasociar macros de esta colección
        let mut macros = self.macros.write().unwrap();
        for instance in macros.values_mut() {
            if instance.collection_id.as_deref() == Some(collection_id) {
                instance.collection_id = None;
            }
        }

        log::info!("Collection deleted: {}", collection_id);
        true
    }

    pub fn enable_collection(&self, collection_id: &str) -> bool {
        let collections = self.collections.read().unwrap();
        if !collections.contains_key(collection_id) {
            log::warn!(
                "enable_collection: collection '{}' not found",
                collection_id
            );
            return false;
        }
        drop(collections);

        if let Err(e) = self
            .persistence
            .collection_repo
            .set_enabled(collection_id, true)
        {
            log::error!("Failed to persist collection enable: {}", e);
            return false;
        }

        // Habilitar todas las macros de la colección
        let mut macros = self.macros.write().unwrap();
        for instance in macros.values_mut() {
            if instance.collection_id.as_deref() == Some(collection_id) {
                instance.enable();
            }
        }

        let mut collections = self.collections.write().unwrap();
        if let Some(collection) = collections.get_mut(collection_id) {
            collection.enabled = true;
        }

        log::info!("Collection enabled: {}", collection_id);
        true
    }

    pub fn disable_collection(&self, collection_id: &str) -> bool {
        let collections = self.collections.read().unwrap();
        if !collections.contains_key(collection_id) {
            log::warn!(
                "disable_collection: collection '{}' not found",
                collection_id
            );
            return false;
        }
        drop(collections);

        if let Err(e) = self
            .persistence
            .collection_repo
            .set_enabled(collection_id, false)
        {
            log::error!("Failed to persist collection disable: {}", e);
            return false;
        }

        // Deshabilitar todas las macros de la colección
        let mut macros = self.macros.write().unwrap();
        for instance in macros.values_mut() {
            if instance.collection_id.as_deref() == Some(collection_id) {
                instance.disable();
            }
        }

        let mut collections = self.collections.write().unwrap();
        if let Some(collection) = collections.get_mut(collection_id) {
            collection.enabled = false;
        }

        log::info!("Collection disabled: {}", collection_id);
        true
    }

    pub fn list_collections(&self) -> Vec<CollectionView> {
        let collections = self.collections.read().unwrap();
        let macros = self.macros.read().unwrap();

        let mut views: Vec<CollectionView> = collections
            .values()
            .map(|c| {
                let macros_count = macros
                    .values()
                    .filter(|m| m.collection_id.as_deref() == Some(&c.id))
                    .count();
                CollectionView {
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
                .persistence
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

        if let Err(e) = self.persistence.collection_repo.reorder(&ordered_ids) {
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
    ) -> bool {
        let mut collections = self.collections.write().unwrap();
        if let Some(collection) = collections.get_mut(collection_id) {
            if let Some(old_hotkey_str) = &collection.toggle_enabled_hotkey {
                if let Some(old_hotkey_vk) = self.key_mapper.string_to_vk(old_hotkey_str) {
                    let mut toggle_map = self.toggle_hotkeys.write().unwrap();
                    toggle_map.remove(&old_hotkey_vk);
                }
            }

            let new_hotkey_vk = hotkey
                .as_ref()
                .and_then(|key| self.key_mapper.string_to_vk(key));
            collection.toggle_enabled_hotkey = hotkey.clone();

            if let Some(new_vk) = new_hotkey_vk {
                let mut toggle_map = self.toggle_hotkeys.write().unwrap();
                toggle_map.insert(new_vk, ToggleTarget::Collection(collection_id.to_string()));
            }

            drop(collections);

            let hotkey_str = hotkey.clone();
            if let Err(e) = self
                .persistence
                .collection_repo
                .set_toggle_hotkey(collection_id, hotkey)
            {
                log::error!(
                    "Failed to persist toggle hotkey for collection '{}': {}",
                    collection_id,
                    e
                );
                return false;
            }

            log::info!(
                "Collection '{}' toggle hotkey set to {:?}",
                collection_id,
                hotkey_str
            );
            true
        } else {
            log::warn!(
                "set_collection_toggle_hotkey: collection '{}' not found",
                collection_id
            );
            false
        }
    }

    fn load_collections(&self) {
        let collections = self.persistence.collection_repo.load_all();
        let mut coll_map = self.collections.write().unwrap();
        let mut toggle_map = self.toggle_hotkeys.write().unwrap();

        for collection in collections {
            if let Some(hotkey_str) = &collection.toggle_enabled_hotkey {
                if let Some(vk) = self.key_mapper.string_to_vk(hotkey_str) {
                    toggle_map.insert(vk, ToggleTarget::Collection(collection.id.clone()));
                }
            }
            coll_map.insert(collection.id.clone(), collection);
        }
        log::info!("Loaded {} collections from persistence", coll_map.len());
        drop(coll_map);
        drop(toggle_map);
    }

    pub fn clear_collections(&self) {
        let mut collections = self.collections.write().unwrap();
        collections.clear();
    }
}
