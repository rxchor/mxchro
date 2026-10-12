use polodb_core::Database;
use std::path::PathBuf;

mod macro_repository;
mod collection_repository;

pub use macro_repository::MacroRepository;
pub use collection_repository::CollectionRepository;

pub struct Persistence {
    pub macro_repo: MacroRepository,
    pub collection_repo: CollectionRepository,
}

impl Persistence {
    pub fn new() -> Self {
        let db_path = Self::get_db_path();

        log::info!("Opening PoloDB at: {:?}", db_path);
        let db = Database::open_path(&db_path).expect("Failed to open PoloDB database");

        Persistence {
            macro_repo: MacroRepository::new(db.clone()),
            collection_repo: CollectionRepository::new(db),
        }
    }

    fn get_db_path() -> PathBuf {
        // Intentar leer desde .env
        if let Ok(path) = std::env::var("MACROS_FILE_PATH") {
            return PathBuf::from(path).with_extension("polodb");
        }

        PathBuf::from("macros.polodb")
    }

    pub fn wipe_data(&self) -> Result<(), String> {
        self.macro_repo.wipe_data()?;
        self.collection_repo.wipe_data()?;
        Ok(())
    }
}
