use polodb_core::{CollectionT, Database};
use std::path::PathBuf;

mod macro_repository;
mod collection_repository;

pub use macro_repository::MacroRepository;
pub use collection_repository::CollectionRepository;

// Schema version - increment when making breaking changes to data structure
const SCHEMA_VERSION: u32 = 1;

pub struct Persistence {
    pub macro_repo: MacroRepository,
    pub collection_repo: CollectionRepository,
}

impl Persistence {
    pub fn new() -> Self {
        let db_path = Self::get_db_path();

        log::info!("Opening PoloDB at: {:?}", db_path);
        let db = Database::open_path(&db_path).expect("Failed to open PoloDB database");

        // Check if we need to reset due to schema version change
        if Self::should_reset_db(&db) {
            log::warn!("Schema version mismatch detected, wiping database...");
            Self::wipe_all_collections(&db);
            Self::set_schema_version(&db);
        }

        Persistence {
            macro_repo: MacroRepository::new(db.clone()),
            collection_repo: CollectionRepository::new(db.clone()),
        }
    }

    fn get_db_path() -> PathBuf {
        // Intentar leer desde .env
        if let Ok(path) = std::env::var("MACROS_FILE_PATH") {
            return PathBuf::from(path).with_extension("polodb");
        }

        PathBuf::from("macros.polodb")
    }

    fn should_reset_db(db: &Database) -> bool {
        // Check if RESET_DB env var is set (for manual reset during development)
        if std::env::var("RESET_DB").map(|v| v == "true" || v == "1").unwrap_or(false) {
            log::info!("RESET_DB environment variable is set, forcing database reset");
            return true;
        }

        // Check schema version
        let meta_collection = db.collection::<polodb_core::bson::Document>("_meta");
        match meta_collection.find_one(polodb_core::bson::doc! { "key": "schema_version" }) {
            Ok(Some(doc)) => {
                if let Some(version) = doc.get_i32("value").ok() {
                    version as u32 != SCHEMA_VERSION
                } else {
                    true
                }
            }
            Ok(None) => true, // No version found, assume old schema
            Err(_) => true,    // Error reading, assume old schema
        }
    }

    fn wipe_all_collections(db: &Database) {
        // Wipe all collections
        let collections = ["macros", "collections"];
        for collection_name in collections {
            let collection = db.collection::<polodb_core::bson::Document>(collection_name);
            if let Err(e) = collection.delete_many(polodb_core::bson::doc! {}) {
                log::error!("Failed to wipe collection {}: {}", collection_name, e);
            }
        }
    }

    fn set_schema_version(db: &Database) {
        let meta_collection = db.collection::<polodb_core::bson::Document>("_meta");
        let doc = polodb_core::bson::doc! {
            "key": "schema_version",
            "value": SCHEMA_VERSION as i32
        };
        
        // Delete existing version if any
        let _ = meta_collection.delete_one(polodb_core::bson::doc! { "key": "schema_version" });
        
        // Insert new version
        if let Err(e) = meta_collection.insert_one(doc) {
            log::error!("Failed to set schema version: {}", e);
        } else {
            log::info!("Schema version set to {}", SCHEMA_VERSION);
        }
    }

    pub fn wipe_data(&self) -> Result<(), String> {
        self.macro_repo.wipe_data()?;
        self.collection_repo.wipe_data()?;
        Ok(())
    }
}
