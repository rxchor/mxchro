use tauri::command;

use crate::{adapters::commands::MANAGERS, core::types::CollectionView};

// ── Collections ─────────────────────────────────────────────

#[command]
pub fn create_collection(name: String) -> Option<String> {
    MANAGERS.collection_manager.create_collection(name)
}

#[command]
pub fn delete_collection(collection_id: String) -> bool {
    MANAGERS
        .collection_manager
        .delete_collection(&collection_id)
}

#[command]
pub fn enable_collection(collection_id: String) -> bool {
    MANAGERS
        .collection_manager
        .enable_collection(&collection_id)
}

#[command]
pub fn disable_collection(collection_id: String) -> bool {
    MANAGERS
        .collection_manager
        .disable_collection(&collection_id)
}

#[command]
pub fn list_collections() -> Vec<CollectionView> {
    MANAGERS.collection_manager.list_collections()
}

#[command]
pub fn rename_collection(collection_id: String, name: String) -> bool {
    MANAGERS
        .collection_manager
        .rename_collection(&collection_id, name)
}

#[command]
pub fn reorder_collections(ordered_ids: Vec<String>) -> bool {
    MANAGERS.collection_manager.reorder_collections(ordered_ids)
}

#[command]
pub fn set_collection_toggle_hotkey(collection_id: String, hotkey: Option<String>) -> bool {
    MANAGERS
        .collection_manager
        .set_collection_toggle_hotkey(&collection_id, hotkey)
}
