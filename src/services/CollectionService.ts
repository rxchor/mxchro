import { invoke } from "@tauri-apps/api/core";
import { Collection, CollectionView } from "@/model/Collection";

// ── Response types (snake_case desde backend) ───────────────

interface CollectionViewResponse {
    id: string;
    name: string;
    enabled: boolean;
    macros_count: number;
    order: number;
    toggle_enabled_hotkey: string | null;
}

// ── Conversion functions ────────────────────────────────────

function fromSnakeCase(response: CollectionViewResponse): CollectionView {
    return {
        id: response.id,
        name: response.name,
        enabled: response.enabled,
        macrosCount: response.macros_count,
        order: response.order,
        toggleEnabledHotkey: response.toggle_enabled_hotkey,
    };
}

// ── Service functions ───────────────────────────────────────

export async function listCollections(): Promise<CollectionView[]> {
    const responses = await invoke<CollectionViewResponse[]>("list_collections");
    return responses.map(fromSnakeCase);
}

export async function createCollection(name: string): Promise<Collection | null> {
    const id = await invoke<string | null>("create_collection", { name });
    if (!id) return null;
    return { id, name, enabled: true, order: 0, toggleEnabledHotkey: null };
}

export async function deleteCollection(collectionId: string): Promise<boolean> {
    return await invoke<boolean>("delete_collection", { collectionId });
}

export async function enableCollection(collectionId: string): Promise<boolean> {
    return await invoke<boolean>("enable_collection", { collectionId });
}

export async function disableCollection(collectionId: string): Promise<boolean> {
    return await invoke<boolean>("disable_collection", { collectionId });
}

export async function updateCollection(collectionId: string, name: string): Promise<boolean> {
    return await invoke<boolean>("rename_collection", { collectionId, name });
}

export async function reorderCollections(orderedIds: string[]): Promise<boolean> {
    return await invoke<boolean>("reorder_collections", { orderedIds });
}

export async function moveMacroToCollection(triggerKey: string, collectionId: string | null): Promise<boolean> {
    return await invoke<boolean>("move_macro_to_collection", { triggerKey, collectionId });
}

export async function setCollectionToggleHotkey(collectionId: string, hotkey: string | null): Promise<boolean> {
    return await invoke<boolean>("set_collection_toggle_hotkey", { collectionId, hotkey });
}
