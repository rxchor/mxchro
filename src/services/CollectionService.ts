import { invoke } from "@tauri-apps/api/core";
import { Collection, CollectionView } from "@/model/Collection";

// ── Response types (snake_case desde backend) ───────────────

interface CollectionViewResponse {
    id: string;
    name: string;
    enabled: boolean;
    macros_count: number;
}

// ── Conversion functions ────────────────────────────────────

function fromSnakeCase(response: CollectionViewResponse): CollectionView {
    return {
        id: response.id,
        name: response.name,
        enabled: response.enabled,
        macrosCount: response.macros_count,
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
    return { id, name, enabled: true };
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

export async function moveMacroToCollection(triggerKey: string, collectionId: string | null): Promise<boolean> {
    return await invoke<boolean>("move_macro_to_collection", { triggerKey, collectionId });
}
