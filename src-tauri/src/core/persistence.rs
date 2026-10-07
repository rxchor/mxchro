use std::fs;
use std::path::PathBuf;

use crate::core::types::MacroConfig;

pub struct Persistence {
    file_path: PathBuf,
}

impl Persistence {
    pub fn new() -> Self {
        let file_path = Self::get_file_path();
        log::info!("Using macros file: {:?}", file_path);
        Persistence { file_path }
    }

    fn get_file_path() -> PathBuf {
        // Intentar leer desde .env
        if let Ok(path) = std::env::var("MACROS_FILE_PATH") {
            return PathBuf::from(path);
        }

        // Producción: %APPDATA%/mxchro/macros.json
        if let Ok(app_data) = std::env::var("APPDATA") {
            let mxchro_dir = PathBuf::from(app_data).join("mxchro");
            // Crear directorio si no existe
            let _ = fs::create_dir_all(&mxchro_dir);
            return mxchro_dir.join("macros.json");
        }

        // Fallback: directorio actual
        PathBuf::from("macros.json")
    }

    pub fn load_macros(&self) -> Vec<MacroConfig> {
        if !self.file_path.exists() {
            log::info!("No macros file found, starting with empty list");
            return Vec::new();
        }

        match fs::read_to_string(&self.file_path) {
            Ok(contents) => match serde_json::from_str::<Vec<MacroConfig>>(&contents) {
                Ok(macros) => {
                    log::info!("Loaded {} macros from file", macros.len());
                    macros
                }
                Err(e) => {
                    log::error!("Failed to parse macros file: {}", e);
                    Vec::new()
                }
            },
            Err(e) => {
                log::error!("Failed to read macros file: {}", e);
                Vec::new()
            }
        }
    }

    pub fn save_macros(&self, macros: &[MacroConfig]) -> Result<(), String> {
        match serde_json::to_string_pretty(macros) {
            Ok(contents) => match fs::write(&self.file_path, contents) {
                Ok(_) => {
                    log::info!("Saved {} macros to file", macros.len());
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to write macros file: {}", e);
                    Err(format!("Failed to write file: {}", e))
                }
            },
            Err(e) => {
                log::error!("Failed to serialize macros: {}", e);
                Err(format!("Failed to serialize: {}", e))
            }
        }
    }

    pub fn wipe_data(&self) -> Result<(), String> {
        if self.file_path.exists() {
            match fs::remove_file(&self.file_path) {
                Ok(_) => {
                    log::info!("Wiped macros file");
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to wipe macros file: {}", e);
                    Err(format!("Failed to wipe file: {}", e))
                }
            }
        } else {
            log::info!("No macros file to wipe");
            Ok(())
        }
    }
}
