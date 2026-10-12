use polodb_core::{CollectionT, Database};
use polodb_core::bson::doc;
use serde::{Deserialize, Serialize};

use crate::core::types::Collection;

/// Representación en BD de una colección. Usa id como _id.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollectionDocument {
    _id: String,
    name: String,
    enabled: bool,
}

impl From<Collection> for CollectionDocument {
    fn from(c: Collection) -> Self {
        CollectionDocument {
            _id: c.id,
            name: c.name,
            enabled: c.enabled,
        }
    }
}

impl From<CollectionDocument> for Collection {
    fn from(doc: CollectionDocument) -> Self {
        Collection {
            id: doc._id,
            name: doc.name,
            enabled: doc.enabled,
        }
    }
}

pub struct CollectionRepository {
    db: Database,
}

impl CollectionRepository {
    pub fn new(db: Database) -> Self {
        CollectionRepository { db }
    }

    fn collections_collection(&self) -> polodb_core::Collection<CollectionDocument> {
        self.db.collection::<CollectionDocument>("collections")
    }

    pub fn load_all(&self) -> Vec<Collection> {
        let collection = self.collections_collection();
        match collection.find(doc! {}).run() {
            Ok(cursor) => {
                let collections: Vec<Collection> = cursor
                    .filter_map(|r| r.ok())
                    .map(Collection::from)
                    .collect();
                log::info!("Loaded {} collections from PoloDB", collections.len());
                collections
            }
            Err(e) => {
                log::error!("Failed to load collections from PoloDB: {}", e);
                Vec::new()
            }
        }
    }

    pub fn insert(&self, collection: &Collection) -> Result<(), String> {
        let coll = self.collections_collection();
        let doc = CollectionDocument::from(collection.clone());
        coll.insert_one(doc).map_err(|e| {
            log::error!("Failed to insert collection '{}': {}", collection.id, e);
            format!("Failed to insert collection: {}", e)
        })?;
        log::info!("Inserted collection '{}' into PoloDB", collection.id);
        Ok(())
    }

    pub fn delete(&self, collection_id: &str) -> Result<(), String> {
        let coll = self.collections_collection();
        coll.delete_one(doc! { "_id": collection_id }).map_err(|e| {
            log::error!("Failed to delete collection '{}': {}", collection_id, e);
            format!("Failed to delete collection: {}", e)
        })?;
        
        // Desasociar todos los macros de esta colección
        let macros_coll = self.db.collection::<polodb_core::bson::Document>("macros");
        macros_coll.update_many(
            doc! { "collection_id": collection_id },
            doc! { "$unset": { "collection_id": "" } },
        ).map_err(|e| {
            log::error!("Failed to disassociate macros from collection '{}': {}", collection_id, e);
            format!("Failed to disassociate macros: {}", e)
        })?;
        
        log::info!("Deleted collection '{}' from PoloDB", collection_id);
        Ok(())
    }

    pub fn set_enabled(&self, collection_id: &str, enabled: bool) -> Result<(), String> {
        let coll = self.db.collection::<polodb_core::bson::Document>("collections");
        coll.update_one(
            doc! { "_id": collection_id },
            doc! { "$set": { "enabled": enabled } },
        ).map_err(|e| {
            log::error!("Failed to set enabled={} for collection '{}': {}", enabled, collection_id, e);
            format!("Failed to update collection: {}", e)
        })?;
        log::info!("Collection '{}' enabled={}", collection_id, enabled);
        Ok(())
    }

    pub fn update(&self, collection: &Collection) -> Result<(), String> {
        let coll = self.db.collection::<polodb_core::bson::Document>("collections");
        coll.update_one(
            doc! { "_id": &collection.id },
            doc! { "$set": { "name": &collection.name, "enabled": collection.enabled } },
        ).map_err(|e| {
            log::error!("Failed to update collection '{}': {}", collection.id, e);
            format!("Failed to update collection: {}", e)
        })?;
        log::info!("Updated collection '{}'", collection.id);
        Ok(())
    }

    pub fn wipe_data(&self) -> Result<(), String> {
        let collection = self.collections_collection();
        collection.delete_many(doc! {}).map_err(|e| {
            log::error!("Failed to wipe collections from PoloDB: {}", e);
            format!("Failed to wipe collections: {}", e)
        })?;
        log::info!("Wiped all collections from PoloDB");
        Ok(())
    }
}
