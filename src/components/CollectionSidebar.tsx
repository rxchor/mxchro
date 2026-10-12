import { useState } from "react";
import { type CollectionView } from "@/model/Collection";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { FolderOpen, Plus, Trash2, Inbox } from "lucide-react";
import { cn } from "@/lib/utils";

const UNCATEGORIZED_ID = "__uncategorized__";

interface CollectionSidebarProps {
  collections: CollectionView[];
  selectedCollectionId: string | null;
  onSelectCollection: (id: string | null) => void;
  onCreateCollection: (name: string) => void;
  onDeleteCollection: (id: string) => void;
}

export function CollectionSidebar({
  collections,
  selectedCollectionId,
  onSelectCollection,
  onCreateCollection,
  onDeleteCollection,
}: CollectionSidebarProps) {
  const [newCollectionName, setNewCollectionName] = useState("");
  const [showInput, setShowInput] = useState(false);

  const handleCreate = () => {
    const trimmed = newCollectionName.trim();
    if (trimmed) {
      onCreateCollection(trimmed);
      setNewCollectionName("");
      setShowInput(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleCreate();
    } else if (e.key === "Escape") {
      setNewCollectionName("");
      setShowInput(false);
    }
  };

  const sortedCollections = [...collections].sort((a, b) =>
    a.name.localeCompare(b.name)
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
            onKeyDown={handleKeyDown}
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
            selectedCollectionId === null && "bg-accent font-medium"
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
            selectedCollectionId === UNCATEGORIZED_ID && "bg-accent font-medium"
          )}
          onClick={() => onSelectCollection(UNCATEGORIZED_ID)}
        >
          <Inbox className="size-4 shrink-0" />
          <span className="flex-1 text-left">Uncategorized</span>
        </button>

        {/* Collections list */}
        <div className="flex flex-col gap-0.5 mt-2">
          {sortedCollections.map((collection) => (
            <div
              key={collection.id}
              className={cn(
                "group flex items-center gap-2 px-2 py-1.5 rounded-md text-sm transition-colors",
                "hover:bg-accent",
                selectedCollectionId === collection.id && "bg-accent font-medium"
              )}
            >
              <button
                className="flex items-center gap-2 flex-1 min-w-0"
                onClick={() => onSelectCollection(collection.id)}
              >
                <FolderOpen className="size-4 shrink-0" />
                <span className="flex-1 text-left truncate">
                  {collection.name}
                </span>
                <span className="text-xs text-muted-foreground">
                  {collection.macrosCount}
                </span>
              </button>
              <Button
                variant="ghost"
                size="icon"
                className="size-5 opacity-0 group-hover:opacity-100 transition-opacity"
                onClick={(e) => {
                  e.stopPropagation();
                  if (confirm(`Delete collection "${collection.name}"?`)) {
                    onDeleteCollection(collection.id);
                  }
                }}
              >
                <Trash2 className="size-3" />
              </Button>
            </div>
          ))}
        </div>
      </nav>
    </div>
  );
}

export { UNCATEGORIZED_ID };
