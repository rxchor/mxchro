"use client";

import {
  DndContext,
  closestCenter,
  DragOverlay,
  KeyboardSensor,
  MouseSensor,
  TouchSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  arrayMove,
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { CSSProperties, useState } from "react";
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import { ScrollArea } from "@/components/ui/scroll-area";
import { KeyEvent } from "@/model/KeyEvent";
import { formattedKey } from "@/services/KeyListService";

type Item = { id: string; action: KeyEvent };

function SortableItem({ id, action }: Item) {
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
    transition,
  };

  // TODO: refactor and turn this into a separate component
  return (
    <div ref={setNodeRef} style={style} {...attributes} {...listeners}>
      <MacroCard
        className={cn(
          "p-3 mb-2 cursor-grab border-2",
          isDragging && "opacity-0"
        )}
        action={action}
      />
    </div>
  );
}

export default function MacroList() {
  const [items, setItems] = useState<Item[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(MouseSensor),
    useSensor(TouchSensor),
    useSensor(KeyboardSensor)
  );

  return (
    <>
      <div className="row-span-2 rounded-md bg-primary-foreground p-5 flex flex-col justify-between gap-5 h-full w-full">
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragStart={({ active }) => setActiveId(active.id as string)}
          onDragEnd={({ active, over }) => {
            setActiveId(null);
            if (over && active.id !== over.id) {
              setItems((items) => {
                const oldIndex = items.findIndex((i) => i.id === active.id);
                const newIndex = items.findIndex((i) => i.id === over.id);
                return arrayMove(items, oldIndex, newIndex);
              });
            }
          }}
          onDragCancel={() => setActiveId(null)}
        >
          {/* Sortable items */}
          <SortableContext
            items={items.map((i) => i.id)}
            strategy={verticalListSortingStrategy}
          >
            <ScrollArea>
              <div className="flex flex-col">
                {items.map((item) => (
                  <SortableItem key={item.id} {...item} />
                ))}
              </div>
            </ScrollArea>
          </SortableContext>

          {/* Overlay for dragging item */}
          <DragOverlay>
            {activeId ? (
              <>
                {(() => {
                  const found = items.find((i) => i.id === activeId)?.action;
                  if (!found) return null;
                  return (
                    <MacroCard
                      action={found}
                      className="border-primary shadow-2xl cursor-grabbing"
                    />
                  );
                })()}
              </>
            ) : null}
          </DragOverlay>
        </DndContext>

        {items.length === 0 && (
          <div className="flex items-center justify-center text-muted-foreground text-sm">
            No actions yet. Add one using the form below.
          </div>
        )}
      </div>

    </>
  );
}

function MacroCard({ className, action, ...props }: React.ComponentProps<"div"> & { action: KeyEvent }) {
  return (
    <Card
      className={cn(
        "p-3 mb-2 cursor-grab border-2 flex flex-row gap-4",
        className
      )}
      {...props}
    >
      {action.key && (
        <div className="font-semibold p-1 pl-2 pr-2 rounded-md bg-primary text-primary-foreground">
          {formattedKey(action.key)}  
        </div>
      )}
      {action.key && action.sleep && <div className="self-center">+</div>}
      {action.sleep && (
        <div
          className="font-semibold p-1 pl-2 pr-2 rounded-md bg-primary text-primary-foreground"
        >
          {action.sleep} ms
        </div>
      )}
    </Card>
  );
}