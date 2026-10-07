import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxGroup,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
  ComboboxTrigger,
  ComboboxData,
} from "@/components/ui/shadcn-io/combobox";
import { KeyboardIcon, Plus, TimerIcon, ArrowDownToLine, ArrowUpFromLine, ArrowUpDown } from "lucide-react";
import { ActionConfig } from "@/model/ActionConfig";
import { getKeys, formattedKey } from "@/services/KeyListService";
import { cn } from "@/lib/utils";

type ActionType = "key" | "wait";
type KeyActionType = "both" | "press" | "release";

interface AddActionFormProps {
  onAdd: (actions: ActionConfig[]) => void;
}

export default function AddActionForm({ onAdd }: AddActionFormProps) {
  const [type, setType] = useState<ActionType>("key");
  const [keyActionType, setKeyActionType] = useState<KeyActionType>("both");
  const [key, setKey] = useState("");
  const [sleep, setSleep] = useState("");
  const [keys, setKeys] = useState<ComboboxData[]>([]);

  useEffect(() => {
    setKeys(
      getKeys().map((k) => ({ label: formattedKey(k), value: k }))
    );
  }, []);

  function reset() {
    setKey("");
    setSleep("");
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (type === "key" && key) {
      const actions: ActionConfig[] = [];
      if (keyActionType === "both") {
        actions.push({ type: "key", key, press: true });
        actions.push({ type: "key", key, press: false });
      } else {
        actions.push({ type: "key", key, press: keyActionType === "press" });
      }
      onAdd(actions);
      reset();
      return;
    }
    if (type === "wait") {
      const ms = parseInt(sleep, 10);
      if (!isNaN(ms) && ms > 0) {
        onAdd([{ type: "wait", duration_ms: ms }]);
        reset();
      }
    }
  }

  const sleepMs = parseInt(sleep, 10);
  const canSubmit =
    type === "key"
      ? !!key
      : !isNaN(sleepMs) && sleepMs > 0;

  return (
    <form
      onSubmit={handleSubmit}
      className="bg-primary-foreground rounded-md p-5 flex flex-col gap-4 h-full"
    >
      <h3 className="font-semibold">Add Action</h3>

      <div className="inline-flex w-full rounded-lg bg-muted p-1">
        <button
          type="button"
          onClick={() => setType("key")}
          className={cn(
            "flex-1 inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium transition-all",
            type === "key"
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground"
          )}
        >
          <KeyboardIcon className="size-4" />
          Key
        </button>
        <button
          type="button"
          onClick={() => setType("wait")}
          className={cn(
            "flex-1 inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium transition-all",
            type === "wait"
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground"
          )}
        >
          <TimerIcon className="size-4" />
          Wait
        </button>
      </div>

      {type === "key" ? (
        <>
          <div className="flex flex-col gap-2">
            <Label>Action Type</Label>
            <div className="inline-flex w-full rounded-lg bg-muted p-1">
              <button
                type="button"
                onClick={() => setKeyActionType("both")}
                className={cn(
                  "flex-1 inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-md px-2 py-1 text-sm font-medium transition-all",
                  keyActionType === "both"
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <ArrowUpDown className="size-4" />
                Both
              </button>
              <button
                type="button"
                onClick={() => setKeyActionType("press")}
                className={cn(
                  "flex-1 inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-md px-2 py-1 text-sm font-medium transition-all",
                  keyActionType === "press"
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <ArrowDownToLine className="size-4" />
                Press
              </button>
              <button
                type="button"
                onClick={() => setKeyActionType("release")}
                className={cn(
                  "flex-1 inline-flex items-center justify-center gap-1.5 whitespace-nowrap rounded-md px-2 py-1 text-sm font-medium transition-all",
                  keyActionType === "release"
                    ? "bg-background text-foreground shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                )}
              >
                <ArrowUpFromLine className="size-4" />
                Release
              </button>
            </div>
          </div>
          <div className="flex flex-col gap-2">
            <Label>Key</Label>
            <Combobox
              value={key}
              onValueChange={setKey}
              data={keys}
              type="key"
            >
              <ComboboxTrigger placeholder="Select a key" />
              <ComboboxContent>
                <ComboboxInput placeholder="Search key" />
                <ComboboxEmpty>No key found</ComboboxEmpty>
                <ComboboxList>
                  <ComboboxGroup>
                    {keys.map((k) => (
                      <ComboboxItem key={k.value} value={k.value}>
                        {k.label}
                      </ComboboxItem>
                    ))}
                  </ComboboxGroup>
                </ComboboxList>
              </ComboboxContent>
            </Combobox>
          </div>
        </>
      ) : (
        <div className="flex flex-col gap-2 flex-1">
          <Label htmlFor="sleep">Delay (ms)</Label>
          <Input
            id="sleep"
            type="number"
            min={1}
            value={sleep}
            onChange={(e) => setSleep(e.target.value)}
            placeholder="100"
          />
          <p className="text-xs text-muted-foreground">
            Wait between actions
          </p>
        </div>
      )}

      <Button type="submit" disabled={!canSubmit}>
        <Plus className="size-4" />
        Add
      </Button>
    </form>
  );
}
