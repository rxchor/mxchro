import { CSSProperties, useState } from "react";
import {
  DndContext,
  closestCenter,
  DragOverlay,
  KeyboardSensor,
  MouseSensor,
  TouchSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { KeyEvent } from "@/model/KeyEvent";
import { formattedKey } from "@/services/KeyListService";
import { GripVertical, X } from "lucide-react";

type ActionItem = { id: string; event: KeyEvent };

interface ActionsListProps {
  actions: ActionItem[];
  onReorder: (actions: ActionItem[]) => void;
  onRemove: (id: string) => void;
}

export default function ActionsList({
  actions,
  onReorder,
  onRemove,
}: ActionsListProps) {
  const [activeId, setActiveId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(MouseSensor),
    useSensor(TouchSensor),
    useSensor(KeyboardSensor)
  );

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    setActiveId(null);
    if (!over || active.id === over.id) return;

    const oldIndex = actions.findIndex((a) => a.id === active.id);
    const newIndex = actions.findIndex((a) => a.id === over.id);
    if (oldIndex !== -1 && newIndex !== -1) {
      onReorder(arrayMove(actions, oldIndex, newIndex));
    }
  }

  if (actions.length === 0) {
    return (
      <div className="bg-primary-foreground rounded-md p-5 h-full flex items-center justify-center text-muted-foreground text-sm">
        No actions yet. Use the form on the right to add one.
      </div>
    );
  }

  const activeItem = activeId ? actions.find((a) => a.id === activeId) : null;

  return (
    <div className="bg-primary-foreground rounded-md p-5 h-full flex flex-col">
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragStart={({ active }) => setActiveId(active.id as string)}
        onDragEnd={handleDragEnd}
        onDragCancel={() => setActiveId(null)}
      >
        <SortableContext
          items={actions.map((a) => a.id)}
          strategy={verticalListSortingStrategy}
        >
          <ScrollArea className="flex-1 -mx-1 px-1">
            <div className="flex flex-col gap-2">
              {actions.map((item) => (
                <SortableActionItem
                  key={item.id}
                  id={item.id}
                  action={item.event}
                  onRemove={() => onRemove(item.id)}
                />
              ))}
            </div>
          </ScrollArea>
        </SortableContext>

        <DragOverlay>
          {activeItem ? (
            <ActionCard
              action={activeItem.event}
              className="border-primary shadow-2xl cursor-grabbing rotate-1"
            />
          ) : null}
        </DragOverlay>
      </DndContext>
    </div>
  );
}

function SortableActionItem({
  id,
  action,
  onRemove,
}: {
  id: string;
  action: KeyEvent;
  onRemove: () => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id });

  const style: CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition: isDragging ? undefined : transition,
    opacity: isDragging ? 0 : 1,
  };

  return (
    <div ref={setNodeRef} style={style} className="flex items-center gap-2">
      <button
        type="button"
        {...attributes}
        {...listeners}
        className="cursor-grab touch-none p-1 rounded hover:bg-muted"
      >
        <GripVertical className="size-4 text-muted-foreground" />
      </button>
      <div className="flex-1">
        <ActionCard action={action} />
      </div>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        onClick={onRemove}
        className="size-8"
      >
        <X className="size-4" />
      </Button>
    </div>
  );
}

function ActionCard({
  action,
  className,
}: {
  action: KeyEvent;
  className?: string;
}) {
  const isWait = action.sleep !== undefined && !action.key;
  const isKey = !!action.key;

  return (
    <Card className={cn("p-3 flex items-center gap-3", className)}>
      <span className="text-xs uppercase font-semibold text-muted-foreground w-10">
        {isWait ? "Wait" : isKey ? "Key" : "?"}
      </span>
      {isKey && (
        <span className="font-semibold px-2 py-0.5 rounded bg-primary text-primary-foreground">
          {formattedKey(action.key!)}
        </span>
      )}
      {isWait && <span className="font-semibold">{action.sleep} ms</span>}
      {isKey && action.sleep !== undefined && (
        <span className="text-sm text-muted-foreground">
          hold {action.sleep}ms
        </span>
      )}
    </Card>
  );
}
