import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from "react";
import { type CollectionView } from "@/model/Collection";
import { type MacroView } from "@/model/MacroView";
import { listCollections, createCollection, deleteCollection, enableCollection, disableCollection, updateCollection, reorderCollections, setCollectionToggleHotkey } from "@/services/CollectionService";
import { listMacros } from "@/services/MacroService";

interface CollectionContextType {
  collections: CollectionView[];
  macros: MacroView[];
  selectedCollectionId: string | null;
  loading: boolean;
  selectCollection: (id: string | null) => void;
  handleCreateCollection: (name: string) => Promise<void>;
  handleDeleteCollection: (id: string) => Promise<void>;
  handleToggleCollection: (id: string, enabled: boolean) => Promise<void>;
  handleRenameCollection: (id: string, name: string) => Promise<void>;
  handleReorderCollections: (orderedIds: string[]) => Promise<void>;
  handleSetCollectionToggleHotkey: (id: string, hotkey: string | null) => Promise<void>;
  refreshData: () => Promise<void>;
}

const CollectionContext = createContext<CollectionContextType | undefined>(undefined);

export function CollectionProvider({ children }: { children: ReactNode }) {
  const [collections, setCollections] = useState<CollectionView[]>([]);
  const [macros, setMacros] = useState<MacroView[]>([]);
  const [selectedCollectionId, setSelectedCollectionId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const refreshData = useCallback(async () => {
    setLoading(true);
    try {
      const [collectionsData, macrosData] = await Promise.all([
        listCollections(),
        listMacros(),
      ]);
      setCollections(collectionsData);
      setMacros(macrosData);
    } catch (error) {
      console.error("Error loading data:", error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refreshData();
  }, [refreshData]);

  const selectCollection = (id: string | null) => {
    setSelectedCollectionId(id);
  };

  const handleCreateCollection = async (name: string) => {
    try {
      const newCollection = await createCollection(name);
      if (newCollection) {
        setCollections((prev) => [...prev, { ...newCollection, macrosCount: 0 }]);
      }
    } catch (error) {
      console.error("Error creating collection:", error);
    }
  };

  const handleDeleteCollection = async (id: string) => {
    try {
      const success = await deleteCollection(id);
      if (success) {
        setCollections((prev) => prev.filter((c) => c.id !== id));
        // Mover macros de esta colección a "Uncategorized"
        setMacros((prev) =>
          prev.map((m) =>
            m.collectionId === id ? { ...m, collectionId: null } : m
          )
        );
        // Si la colección eliminada estaba seleccionada, volver a "All"
        if (selectedCollectionId === id) {
          setSelectedCollectionId(null);
        }
      }
    } catch (error) {
      console.error("Error deleting collection:", error);
    }
  };

  const handleToggleCollection = async (id: string, enabled: boolean) => {
    try {
      const success = enabled
        ? await enableCollection(id)
        : await disableCollection(id);
      
      if (success) {
        setCollections((prev) =>
          prev.map((c) => (c.id === id ? { ...c, enabled } : c))
        );
        // Actualizar macros de esta colección
        setMacros((prev) =>
          prev.map((m) =>
            m.collectionId === id ? { ...m, enabled } : m
          )
        );
      }
    } catch (error) {
      console.error("Error toggling collection:", error);
    }
  };

  const handleRenameCollection = async (id: string, name: string) => {
    try {
      const success = await updateCollection(id, name);
      if (success) {
        setCollections((prev) =>
          prev.map((c) => (c.id === id ? { ...c, name } : c))
        );
      }
    } catch (error) {
      console.error("Error renaming collection:", error);
    }
  };

  const handleReorderCollections = async (orderedIds: string[]) => {
    try {
      const success = await reorderCollections(orderedIds);
      if (success) {
        await refreshData();
      }
    } catch (error) {
      console.error("Error reordering collections:", error);
    }
  };

  const handleSetCollectionToggleHotkey = async (id: string, hotkey: string | null) => {
    try {
      const success = await setCollectionToggleHotkey(id, hotkey);
      if (success) {
        await refreshData();
      }
    } catch (error) {
      console.error("Error setting collection toggle hotkey:", error);
    }
  };

  return (
    <CollectionContext.Provider
      value={{
        collections,
        macros,
        selectedCollectionId,
        loading,
        selectCollection,
        handleCreateCollection,
        handleDeleteCollection,
        handleToggleCollection,
        handleRenameCollection,
        handleReorderCollections,
        handleSetCollectionToggleHotkey,
        refreshData,
      }}
    >
      {children}
    </CollectionContext.Provider>
  );
}

export function useCollectionContext() {
  const context = useContext(CollectionContext);
  if (context === undefined) {
    throw new Error("useCollectionContext must be used within a CollectionProvider");
  }
  return context;
}
