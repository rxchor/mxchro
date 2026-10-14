import { useState } from "react";
import { type CollectionView } from "@/model/Collection";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import {
  FolderOpen,
  Plus,
  Trash2,
  Inbox,
  Pencil,
  Check,
  X,
  Keyboard,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { KeyInput } from "@/components/KeyInput";
import { formattedKey } from "@/services/KeyListService";

const UNCATEGORIZED_ID = "__uncategorized__";

interface CollectionSidebarProps {
  collections: CollectionView[];
  selectedCollectionId: string | null;
  onSelectCollection: (id: string | null) => void;
  onCreateCollection: (name: string) => void;
  onDeleteCollection: (id: string) => void;
  onRenameCollection: (id: string, name: string) => void;
  onSetToggleHotkey: (id: string, hotkey: string | null) => void;
}

export function CollectionSidebar({
  collections,
  selectedCollectionId,
  onSelectCollection,
  onCreateCollection,
  onDeleteCollection,
  onRenameCollection,
  onSetToggleHotkey,
}: CollectionSidebarProps) {
  const [newCollectionName, setNewCollectionName] = useState("");
  const [showInput, setShowInput] = useState(false);
  const [editingCollectionId, setEditingCollectionId] = useState<string | null>(
    null,
  );
  const [editingName, setEditingName] = useState("");
  const [editingHotkeyCollectionId, setEditingHotkeyCollectionId] = useState<string | null>(null);

  const handleCreate = () => {
    const trimmed = newCollectionName.trim();
    if (trimmed) {
      onCreateCollection(trimmed);
      setNewCollectionName("");
      setShowInput(false);
    }
  };

  const handleCreateKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleCreate();
    } else if (e.key === "Escape") {
      setNewCollectionName("");
      setShowInput(false);
    }
  };

  const handleStartRename = (collection: CollectionView) => {
    setEditingCollectionId(collection.id);
    setEditingName(collection.name);
  };

  const handleSaveRename = () => {
    if (editingCollectionId && editingName.trim()) {
      onRenameCollection(editingCollectionId, editingName.trim());
      setEditingCollectionId(null);
      setEditingName("");
    }
  };

  const handleCancelRename = () => {
    setEditingCollectionId(null);
    setEditingName("");
  };

  const handleRenameKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleSaveRename();
    } else if (e.key === "Escape") {
      handleCancelRename();
    }
  };

  const sortedCollections = [...collections].sort((a, b) =>
    a.name.localeCompare(b.name),
  );

  return (
    <div className="flex flex-col h-full p-3 gap-2">
      <div className="flex items-center justify-between px-2 py-1">
        <h2 className="text-sm font-semibold">Collections</h2>
        <Button
          variant="ghost"
          size="icon"
          className="size-6"
          onClick={() => setShowInput(!showInput)}
        >
          <Plus className="size-4" />
        </Button>
      </div>

      {showInput && (
        <div className="flex gap-1 px-2">
          <Input
            placeholder="Collection name..."
            value={newCollectionName}
            onChange={(e) => setNewCollectionName(e.target.value)}
            onKeyDown={handleCreateKeyDown}
            className="h-7 text-xs"
            autoFocus
          />
          <Button
            size="sm"
            variant="outline"
            className="h-7 px-2 text-xs"
            onClick={handleCreate}
            disabled={!newCollectionName.trim()}
          >
            Add
          </Button>
        </div>
      )}

      <nav className="flex flex-col gap-1">
        {/* All macros */}
        <button
          className={cn(
            "flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors",
            "hover:bg-accent",
            selectedCollectionId === null && "bg-accent font-medium",
          )}
          onClick={() => onSelectCollection(null)}
        >
          <FolderOpen className="size-4 shrink-0" />
          <span className="flex-1 text-left">All</span>
          <span className="text-xs text-muted-foreground">
            {collections.reduce((sum, c) => sum + c.macrosCount, 0)}
          </span>
        </button>

        {/* Uncategorized */}
        <button
          className={cn(
            "flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors",
            "hover:bg-accent",
            selectedCollectionId === UNCATEGORIZED_ID &&
              "bg-accent font-medium",
          )}
          onClick={() => onSelectCollection(UNCATEGORIZED_ID)}
        >
          <Inbox className="size-4 shrink-0" />
          <span className="flex-1 text-left">Uncategorized</span>
        </button>

        {sortedCollections.length > 0 && (
          <>
            <Separator className="my-1" />

            {/* Collections list */}
            <div className="flex flex-col gap-0.5 mt-2">
              {sortedCollections.map((collection) => {
                const isEditing = editingCollectionId === collection.id;

                return (
                  <div
                    key={collection.id}
                    className={cn(
                      "group flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors",
                      "hover:bg-accent",
                      selectedCollectionId === collection.id &&
                        "bg-accent font-medium",
                    )}
                  >
                    {isEditing ? (
                      <>
                        <Input
                          value={editingName}
                          onChange={(e) => setEditingName(e.target.value)}
                          onKeyDown={handleRenameKeyDown}
                          className="h-6 text-xs flex-1"
                          autoFocus
                        />
                        <Button
                          variant="ghost"
                          size="icon"
                          className="size-5 hover:bg-green-100 hover:text-green-700"
                          onClick={handleSaveRename}
                          disabled={!editingName.trim()}
                        >
                          <Check className="size-3" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="size-5 hover:bg-red-100 hover:text-red-700"
                          onClick={handleCancelRename}
                        >
                          <X className="size-3" />
                        </Button>
                      </>
                    ) : editingHotkeyCollectionId === collection.id ? (
                      <>
                        <div className="flex-1 min-w-0">
                          <KeyInput
                            value={collection.toggleEnabledHotkey}
                            onChange={(key) => {
                              onSetToggleHotkey(collection.id, key);
                              setEditingHotkeyCollectionId(null);
                            }}
                            placeholder="Press a key..."
                          />
                        </div>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="size-5 hover:bg-red-100 hover:text-red-700"
                          onClick={(e) => {
                            e.stopPropagation();
                            setEditingHotkeyCollectionId(null);
                          }}
                        >
                          <X className="size-3" />
                        </Button>
                      </>
                    ) : (
                      <>
                        <button
                          className="flex items-center gap-2 flex-1 min-w-0"
                          onClick={() => onSelectCollection(collection.id)}
                        >
                          <FolderOpen className="size-4 shrink-0" />
                          <span className="flex-1 text-left truncate">
                            {collection.name}
                          </span>
                          {collection.toggleEnabledHotkey && (
                            <span className="text-xs px-1.5 py-0.5 rounded bg-blue-100 text-blue-700">
                              {formattedKey(collection.toggleEnabledHotkey)}
                            </span>
                          )}
                          <span className="text-xs text-muted-foreground">
                            {collection.macrosCount}
                          </span>
                        </button>
                        <div className="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-5 hover:bg-purple-100 hover:text-purple-700"
                            onClick={(e) => {
                              e.stopPropagation();
                              setEditingHotkeyCollectionId(collection.id);
                            }}
                            title="Set toggle hotkey"
                          >
                            <Keyboard className="size-3" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-5 hover:bg-blue-100 hover:text-blue-700"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleStartRename(collection);
                            }}
                          >
                            <Pencil className="size-3" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="size-5 hover:bg-red-100 hover:text-red-700"
                            onClick={(e) => {
                              e.stopPropagation();
                              if (
                                confirm(
                                  `Delete collection "${collection.name}"?`,
                                )
                              ) {
                                onDeleteCollection(collection.id);
                              }
                            }}
                          >
                            <Trash2 className="size-3" />
                          </Button>
                        </div>
                      </>
                    )}
                  </div>
                );
              })}
            </div>
          </>
        )}
      </nav>
    </div>
  );
}

export { UNCATEGORIZED_ID };
