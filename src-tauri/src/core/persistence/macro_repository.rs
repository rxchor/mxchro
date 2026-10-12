use polodb_core::{CollectionT, Database};
use polodb_core::bson::doc;
use serde::{Deserialize, Serialize};

use crate::core::types::{Action, Macro, Mode};

/// Representación en BD de un macro. Usa trigger_key como _id.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MacroDocument {
    _id: String,
    mode: Mode,
    actions: Vec<Action>,
    block_original_event: bool,
    ignore_injected_keys: bool,
    enabled: bool,
    collection_id: Option<String>,
}

impl From<Macro> for MacroDocument {
    fn from(m: Macro) -> Self {
        MacroDocument {
            _id: m.trigger_key,
            mode: m.mode,
            actions: m.actions,
            block_original_event: m.block_original_event,
            ignore_injected_keys: m.ignore_injected_keys,
            enabled: m.enabled,
            collection_id: m.collection_id,
        }
    }
}

impl From<MacroDocument> for Macro {
    fn from(doc: MacroDocument) -> Self {
        Macro {
            trigger_key: doc._id,
            mode: doc.mode,
            actions: doc.actions,
            block_original_event: doc.block_original_event,
            ignore_injected_keys: doc.ignore_injected_keys,
            enabled: doc.enabled,
            collection_id: doc.collection_id,
        }
    }
}

pub struct MacroRepository {
    db: Database,
}

impl MacroRepository {
    pub fn new(db: Database) -> Self {
        MacroRepository { db }
    }

    fn macros_collection(&self) -> polodb_core::Collection<MacroDocument> {
        self.db.collection::<MacroDocument>("macros")
    }

    pub fn load_all(&self) -> Vec<Macro> {
        let collection = self.macros_collection();
        match collection.find(doc! {}).run() {
            Ok(cursor) => {
                let macros: Vec<Macro> = cursor
                    .filter_map(|r| r.ok())
                    .map(Macro::from)
                    .collect();
                log::info!("Loaded {} macros from PoloDB", macros.len());
                macros
            }
            Err(e) => {
                log::error!("Failed to load macros from PoloDB: {}", e);
                Vec::new()
            }
        }
    }

    pub fn insert(&self, macro_entity: &Macro) -> Result<(), String> {
        let collection = self.macros_collection();
        let doc = MacroDocument::from(macro_entity.clone());
        collection.insert_one(doc).map_err(|e| {
            log::error!("Failed to insert macro '{}': {}", macro_entity.trigger_key, e);
            format!("Failed to insert: {}", e)
        })?;
        log::info!("Inserted macro '{}' into PoloDB", macro_entity.trigger_key);
        Ok(())
    }

    pub fn delete(&self, trigger_key: &str) -> Result<(), String> {
        let collection = self.macros_collection();
        collection.delete_one(doc! { "_id": trigger_key }).map_err(|e| {
            log::error!("Failed to delete macro '{}': {}", trigger_key, e);
            format!("Failed to delete: {}", e)
        })?;
        log::info!("Deleted macro '{}' from PoloDB", trigger_key);
        Ok(())
    }

    pub fn set_enabled(&self, trigger_key: &str, enabled: bool) -> Result<(), String> {
        let collection = self.db.collection::<polodb_core::bson::Document>("macros");
        collection
            .update_one(
                doc! { "_id": trigger_key },
                doc! { "$set": { "enabled": enabled } },
            )
            .map_err(|e| {
                log::error!("Failed to set enabled={} for macro '{}': {}", enabled, trigger_key, e);
                format!("Failed to update: {}", e)
            })?;
        log::info!("Macro '{}' enabled={}", trigger_key, enabled);
        Ok(())
    }

    /// Reemplaza un macro completo (usado cuando la trigger_key puede cambiar).
    pub fn replace(&self, old_trigger_key: &str, macro_entity: &Macro) -> Result<(), String> {
        let collection = self.macros_collection();

        // Borrar el viejo
        collection.delete_one(doc! { "_id": old_trigger_key }).map_err(|e| {
            log::error!("Failed to delete old macro '{}': {}", old_trigger_key, e);
            format!("Failed to delete old: {}", e)
        })?;

        // Insertar el nuevo
        let doc = MacroDocument::from(macro_entity.clone());
        collection.insert_one(doc).map_err(|e| {
            log::error!("Failed to insert new macro '{}': {}", macro_entity.trigger_key, e);
            format!("Failed to insert new: {}", e)
        })?;

        log::info!("Replaced macro '{}' -> '{}'", old_trigger_key, macro_entity.trigger_key);
        Ok(())
    }

    pub fn wipe_data(&self) -> Result<(), String> {
        let collection = self.macros_collection();
        collection.delete_many(doc! {}).map_err(|e| {
            log::error!("Failed to wipe macros from PoloDB: {}", e);
            format!("Failed to wipe macros: {}", e)
        })?;
        log::info!("Wiped all macros from PoloDB");
        Ok(())
    }
}
