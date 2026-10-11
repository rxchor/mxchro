import { CSSProperties, ReactElement, useState } from "react";
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
import { Action } from "@/model/Action";
import { formattedKey } from "@/services/KeyListService";
import {
  GripVertical,
  X,
  ArrowDownToLine,
  ArrowUpFromLine,
  Timer,
} from "lucide-react";

type ActionItem = { id: string; action: Action };

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
    useSensor(KeyboardSensor),
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
      <div className="bg-primary-foreground rounded-md p-5 h-[40vh] flex items-center justify-center text-muted-foreground text-sm">
        No actions yet. Use the form on the right to add one.
      </div>
    );
  }

  const activeItem = activeId ? actions.find((a) => a.id === activeId) : null;

  return (
    <div className="bg-primary-foreground rounded-md p-5 flex flex-col h-[40vh]">
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
          <div className="flex-1 overflow-y-auto">
            <div className="flex flex-col gap-1.5">
              {actions.map((item) => (
                <SortableActionItem
                  key={item.id}
                  id={item.id}
                  action={item.action}
                  onRemove={() => onRemove(item.id)}
                />
              ))}
            </div>
          </div>
        </SortableContext>

        <DragOverlay>
          {activeItem ? (
            <div className="flex items-center gap-2">
              <ActionCard
                action={activeItem.action}
                className="flex-1 border-primary shadow-2xl cursor-grabbing"
              />
              <div className="size-8 shrink-0" />
            </div>
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
  action: Action;
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
      <div
        {...attributes}
        {...listeners}
        className="flex-1 flex items-center gap-2 cursor-grab touch-none min-w-0"
      >
        <ActionCard action={action} className="flex-1" />
      </div>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        onClick={onRemove}
        className="size-8 shrink-0"
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
  action: Action;
  className?: string;
}) {
  function actionIcon(
    action: Action,
    divClassName: string,
    iconClassName: string,
  ): ReactElement {
    return (
      <div className={divClassName}>
        {(() => {
          if (action.type === "key") {
            if (action.press) {
              return <ArrowDownToLine className={iconClassName} />;
            } else {
              return <ArrowUpFromLine className={iconClassName} />;
            }
          } else {
            return <Timer className={iconClassName} />;
          }
        })()}
      </div>
    );
  }

  function actionLabel(action: Action, className: string): ReactElement {
    return (
      <span className={className}>
        {(() => {
          return action.type === "key"
            ? action.press
              ? "Press"
              : "Release"
            : "Wait";
        })()}
      </span>
    );
  }

  function actionText(action: Action): ReactElement {
    if (action.type === "key") {
      return (
        <span className="font-semibold text-sm px-2 py-0.5 rounded bg-primary text-primary-foreground whitespace-nowrap">
          {formattedKey(action.key)}
        </span>
      );
    } else {
      return (
        <span className="font-semibold text-sm">{action.duration_ms} ms</span>
      );
    }
  }

  return (
    <Card
      className={cn(
        "p-2 flex flex-row items-center gap-3",
        className,
      )}
    >
      <GripVertical className="size-4 text-muted-foreground shrink-0" />
      <div className="flex-1 flex flex-row items-center gap-4">
        {actionIcon(
          action,
          "flex items-center justify-center w-8 h-8 rounded bg-primary/10 shrink-0",
          "size-3 text-primary",
        )}
        {actionLabel(
          action,
          "text-xs uppercase font-semibold text-muted-foreground w-16 text-center shrink-0",
        )}
        <div className="flex-1">
          {actionText(action)}
        </div>
      </div>
    </Card>
  );
}
