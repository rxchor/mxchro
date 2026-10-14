export interface Collection {
    id: string;
    name: string;
    enabled: boolean;
    order: number;
    toggleEnabledHotkey: string | null;
}

export interface CollectionView {
    id: string;
    name: string;
    enabled: boolean;
    macrosCount: number;
    order: number;
    toggleEnabledHotkey: string | null;
}
