import { useState, useEffect } from "react";
import { deleteMacro, enableMacro, disableMacro, wipeData } from "@/services/MacroService";
import { moveMacroToCollection } from "@/services/CollectionService";
import { type MacroView } from "@/model/MacroView";
import { type CollectionView } from "@/model/Collection";
import { formattedKey } from "@/services/KeyListService";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { CollectionSidebar, UNCATEGORIZED_ID } from "@/components/CollectionSidebar";
import { CollectionProvider, useCollectionContext } from "@/contexts/CollectionContext";
import { Trash2, RefreshCw, Power, Pencil, Eraser, Folder } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";

export function HomePage() {
  return (
    <CollectionProvider>
      <div className="flex h-full w-full">
        <aside className="w-64 border-r bg-background shrink-0">
          <CollectionSidebarWrapper />
        </aside>
        <main className="flex-1 overflow-auto">
          <HomePageContent />
        </main>
      </div>
    </CollectionProvider>
  );
}

function CollectionSidebarWrapper() {
  const {
    collections,
    selectedCollectionId,
    selectCollection,
    handleCreateCollection,
    handleDeleteCollection,
    handleRenameCollection,
    handleSetCollectionToggleHotkey,
  } = useCollectionContext();

  return (
    <CollectionSidebar
      collections={collections}
      selectedCollectionId={selectedCollectionId}
      onSelectCollection={selectCollection}
      onCreateCollection={handleCreateCollection}
      onDeleteCollection={handleDeleteCollection}
      onRenameCollection={handleRenameCollection}
      onSetToggleHotkey={handleSetCollectionToggleHotkey}
    />
  );
}

function HomePageContent() {
  const { collections, macros, selectedCollectionId, loading, refreshData } = useCollectionContext();
  const [wiping, setWiping] = useState(false);
  const navigate = useNavigate();

  // Listen for toggle events from backend (triggered by toggle hotkeys)
  useEffect(() => {
    const unlisten = listen('toggle-event', () => {
      refreshData();
    });

    return () => {
      unlisten.then(unsub => unsub());
    };
  }, [refreshData]);

  async function handleDelete(triggerKey: string) {
    try {
      const success = await deleteMacro(triggerKey);
      if (success) {
        await refreshData();
      }
    } catch (e) {
      console.error("Error deleting macro:", e);
    }
  }

  async function handleToggleEnabled(triggerKey: string, currentEnabled: boolean) {
    try {
      const success = currentEnabled 
        ? await disableMacro(triggerKey)
        : await enableMacro(triggerKey);
      
      if (success) {
        await refreshData();
      }
    } catch (e) {
      console.error("Error toggling macro:", e);
    }
  }

  async function handleMoveToCollection(triggerKey: string, collectionId: string | null) {
    try {
      const success = await moveMacroToCollection(triggerKey, collectionId);
      if (success) {
        await refreshData();
      }
    } catch (e) {
      console.error("Error moving macro:", e);
    }
  }

  function handleEdit(triggerKey: string) {
    navigate("/create-macro", { state: { editTriggerKey: triggerKey } });
  }

  async function handleWipeData() {
    if (!confirm("Are you sure you want to wipe all macro data? This action cannot be undone.")) {
      return;
    }
    
    setWiping(true);
    try {
      await wipeData();
      await refreshData();
    } catch (e) {
      console.error("Error wiping data:", e);
    } finally {
      setWiping(false);
    }
  }

  // Filter macros based on selected collection
  const filteredMacros = selectedCollectionId === null
    ? macros // All macros
    : selectedCollectionId === UNCATEGORIZED_ID
    ? macros.filter((m) => !m.collectionId) // Uncategorized
    : macros.filter((m) => m.collectionId === selectedCollectionId); // Specific collection

  const sortedCollections = [...collections].sort((a, b) => a.name.localeCompare(b.name));

  // Group macros by collection for "All" view
  const uncategorizedMacros = macros.filter((m) => !m.collectionId);
  const getMacrosByCollection = (collectionId: string) =>
    macros.filter((m) => m.collectionId === collectionId);

  // Get title based on selected collection
  const getTitle = () => {
    if (selectedCollectionId === null) return "All Macros";
    if (selectedCollectionId === UNCATEGORIZED_ID) return "Uncategorized Macros";
    const collection = collections.find((c) => c.id === selectedCollectionId);
    return collection ? `${collection.name} Macros` : "Macros";
  };

  return (
    <div className="w-full max-w-4xl mx-auto p-6 flex flex-col gap-6">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="font-semibold text-2xl">{getTitle()}</h1>
          <p className="text-sm text-muted-foreground">
            {filteredMacros.length} macro{filteredMacros.length !== 1 ? "s" : ""}
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={refreshData} disabled={loading}>
            <RefreshCw className={`size-4 mr-2 ${loading ? "animate-spin" : ""}`} />
            Refresh
          </Button>
          <Button variant="destructive" size="sm" onClick={handleWipeData} disabled={wiping}>
            <Eraser className={`size-4 mr-2 ${wiping ? "animate-pulse" : ""}`} />
            {wiping ? "Wiping..." : "Wipe Data"}
          </Button>
        </div>
      </header>

      {loading && macros.length === 0 ? (
        <div className="bg-primary-foreground rounded-md p-8 text-center text-muted-foreground">
          Loading macros...
        </div>
      ) : filteredMacros.length === 0 ? (
        <div className="bg-primary-foreground rounded-md p-8 text-center text-muted-foreground">
          {selectedCollectionId === null
            ? "No macros yet. Go to \"Create Macro\" to add one."
            : "No macros in this collection."}
        </div>
      ) : selectedCollectionId === null ? (
        // "All" view: show macros grouped by collection
        <div className="flex flex-col gap-6">
          {/* Macros with collections */}
          {sortedCollections.map((collection) => {
            const collectionMacros = getMacrosByCollection(collection.id);
            if (collectionMacros.length === 0) return null;

            return (
              <div key={collection.id} className="rounded-lg border-2 border-primary/30 bg-primary-foreground p-4">
                <div className="flex items-center gap-2 mb-3 pb-2 border-b border-primary/20">
                  <h2 className="text-sm font-bold text-primary uppercase tracking-wider">
                    {collection.name}
                  </h2>
                  <span className="text-xs font-semibold px-2 py-0.5 rounded-full bg-primary/10 text-primary">
                    {collectionMacros.length}
                  </span>
                </div>
                <div className="flex flex-col gap-2">
                  {collectionMacros.map((macro) => (
                    <MacroCard
                      key={macro.triggerKey}
                      macro={macro}
                      collections={sortedCollections}
                      onEdit={handleEdit}
                      onToggle={handleToggleEnabled}
                      onDelete={handleDelete}
                      onMove={handleMoveToCollection}
                    />
                  ))}
                </div>
              </div>
            );
          })}

          {/* Uncategorized macros */}
          {uncategorizedMacros.length > 0 && (
            <div className="rounded-lg border-2 border-muted bg-muted/30 p-4">
              <div className="flex items-center gap-2 mb-3 pb-2 border-b border-muted">
                <h2 className="text-sm font-bold text-muted-foreground uppercase tracking-wider">
                  Uncategorized
                </h2>
                <span className="text-xs font-semibold px-2 py-0.5 rounded-full bg-muted text-muted-foreground">
                  {uncategorizedMacros.length}
                </span>
              </div>
              <div className="flex flex-col gap-2">
                {uncategorizedMacros.map((macro) => (
                  <MacroCard
                    key={macro.triggerKey}
                    macro={macro}
                    collections={sortedCollections}
                    onEdit={handleEdit}
                    onToggle={handleToggleEnabled}
                    onDelete={handleDelete}
                    onMove={handleMoveToCollection}
                  />
                ))}
              </div>
            </div>
          )}
        </div>
      ) : (
        // Single collection view
        <div className="flex flex-col gap-1.5">
          {filteredMacros.map((macro) => (
            <MacroCard
              key={macro.triggerKey}
              macro={macro}
              collections={sortedCollections}
              onEdit={handleEdit}
              onToggle={handleToggleEnabled}
              onDelete={handleDelete}
              onMove={handleMoveToCollection}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// ── MacroCard component ─────────────────────────────────────

interface MacroCardProps {
  macro: MacroView;
  collections: CollectionView[];
  onEdit: (triggerKey: string) => void;
  onToggle: (triggerKey: string, enabled: boolean) => void;
  onDelete: (triggerKey: string) => void;
  onMove: (triggerKey: string, collectionId: string | null) => void;
}

function MacroCard({ macro, collections, onEdit, onToggle, onDelete, onMove }: MacroCardProps) {
  const [showMoveMenu, setShowMoveMenu] = useState(false);

  return (
    <Card className="p-2.5 flex items-center justify-between">
      <div className="flex items-center gap-3">
        <div className="flex flex-col">
          <span className="font-semibold text-sm">
            {formattedKey(macro.triggerKey)}
          </span>
          <span className="text-xs text-muted-foreground">
            {macro.mode} · {macro.actionsCount} action{macro.actionsCount !== 1 ? "s" : ""}
          </span>
        </div>
        <div className="flex gap-1.5">
          {macro.blockOriginalEvent && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
              Blocks
            </span>
          )}
          <span className={`text-[10px] px-1.5 py-0.5 rounded ${macro.enabled ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
            {macro.enabled ? 'Enabled' : 'Disabled'}
          </span>
        </div>
      </div>
      <div className="flex items-center gap-1">
        {/* Move to collection dropdown */}
        <div className="relative">
          <Button
            variant="ghost"
            size="icon"
            className="size-7"
            onClick={() => setShowMoveMenu(!showMoveMenu)}
            title="Move to collection"
          >
            <Folder className="size-3.5" />
          </Button>
          {showMoveMenu && (
            <>
              <div className="fixed inset-0 z-40" onClick={() => setShowMoveMenu(false)} />
              <div className="absolute right-0 top-full mt-1 z-50 bg-popover border border-border rounded-md shadow-md py-1 min-w-[160px]">
                {macro.collectionId && (
                  <button
                    className="w-full text-left px-3 py-1.5 text-xs hover:bg-accent"
                    onClick={() => { onMove(macro.triggerKey, null); setShowMoveMenu(false); }}
                  >
                    ← Uncategorized
                  </button>
                )}
                {collections
                  .filter((c) => c.id !== macro.collectionId)
                  .map((c) => (
                    <button
                      key={c.id}
                      className="w-full text-left px-3 py-1.5 text-xs hover:bg-accent"
                      onClick={() => { onMove(macro.triggerKey, c.id); setShowMoveMenu(false); }}
                    >
                      → {c.name}
                    </button>
                  ))}
                {collections.filter((c) => c.id !== macro.collectionId).length === 0 && !macro.collectionId && (
                  <span className="px-3 py-1.5 text-xs text-muted-foreground">No collections</span>
                )}
              </div>
            </>
          )}
        </div>
        <Button
          variant="ghost"
          size="icon"
          className="size-7"
          onClick={() => onEdit(macro.triggerKey)}
          title="Edit macro"
        >
          <Pencil className="size-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="size-7"
          onClick={() => onToggle(macro.triggerKey, macro.enabled)}
          title={macro.enabled ? "Disable" : "Enable"}
        >
          <Power className={`size-3.5 ${macro.enabled ? "text-green-600" : "text-red-400"}`} />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="size-7 text-destructive hover:text-destructive"
          onClick={() => onDelete(macro.triggerKey)}
          title="Delete macro"
        >
          <Trash2 className="size-3.5" />
        </Button>
      </div>
    </Card>
  );
}
