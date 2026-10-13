import { Mode } from "@/model/Mode";
import { CollectionView } from "@/model/Collection";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
import { Checkbox } from "@/components/ui/checkbox";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { getKeys, formattedKey } from "@/services/KeyListService";
import { useEffect, useState } from "react";
import { z } from "zod";

export const macroFormSchema = z.object({
  triggerKey: z
    .string()
    .min(1, { message: "The macro must have a trigger key" }),
  mode: z.string().min(1, { message: "Please select a mode" }),
  blockOriginalEvent: z.boolean(),
  ignoreInjectedKeys: z.boolean(),
  enabled: z.boolean(),
  collectionId: z.string().nullable(),
});

export type MacroFormValues = z.infer<typeof macroFormSchema>;

interface MacroFormProps {
  values: MacroFormValues;
  onChange: (values: MacroFormValues) => void;
  collections: CollectionView[];
}

export default function MacroForm({
  values,
  onChange,
  collections,
}: MacroFormProps) {
  const [cbKeys, setCbKeys] = useState<ComboboxData[]>([]);

  useEffect(() => {
    setCbKeys(
      getKeys().map((key) => ({
        label: formattedKey(key),
        value: key,
      })),
    );
  }, []);

  const modes = Object.values(Mode);

  function update<K extends keyof MacroFormValues>(
    key: K,
    value: MacroFormValues[K],
  ) {
    onChange({ ...values, [key]: value });
  }

  return (
    <div className="bg-primary-foreground rounded-lg border p-6 space-y-6">
      {/* Trigger & Mode */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="flex flex-col gap-2">
          <Label className="text-sm font-medium">Trigger Key</Label>
          <Combobox
            value={values.triggerKey}
            onValueChange={(v) => update("triggerKey", v)}
            data={cbKeys}
            type="key"
          >
            <ComboboxTrigger placeholder="Select a key" />
            <ComboboxContent className="h-full">
              <ComboboxInput placeholder="Search key" />
              <ComboboxEmpty>No key found</ComboboxEmpty>
              <ComboboxList>
                <ComboboxGroup>
                  {cbKeys.map((key) => (
                    <ComboboxItem key={key.value} value={key.value}>
                      {key.label}
                    </ComboboxItem>
                  ))}
                </ComboboxGroup>
              </ComboboxList>
            </ComboboxContent>
          </Combobox>
          <p className="text-xs text-muted-foreground">
            Key that triggers the macro
          </p>
        </div>

        <div className="flex flex-col gap-2">
          <Label className="text-sm font-medium">Mode</Label>
          <Select value={values.mode} onValueChange={(v) => update("mode", v)}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Select a mode" />
            </SelectTrigger>
            <SelectContent>
              {modes.map((mode) => (
                <SelectItem key={mode} value={mode.toString()}>
                  {mode.charAt(0).toUpperCase() + mode.slice(1).toLowerCase()}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            How the macro executes
          </p>
        </div>
      </div>

      <Separator />

      {/* Collection */}
      <div className="flex flex-col gap-2">
        <Label className="text-sm font-medium">Collection</Label>
        <Select
          value={values.collectionId ?? "none"}
          onValueChange={(v) => update("collectionId", v === "none" ? null : v)}
        >
          <SelectTrigger className="w-full md:w-1/2">
            <SelectValue placeholder="Select a collection" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="none">No collection</SelectItem>
            {collections.map((c) => (
              <SelectItem key={c.id} value={c.id}>
                {c.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <p className="text-xs text-muted-foreground">
          Organize this macro into a collection
        </p>
      </div>

      <Separator />

      {/* Options */}
      <div className="flex flex-col gap-4">
        <Label className="text-sm font-medium">Options</Label>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div
            className={`flex items-center gap-3 p-3 rounded-md border bg-background cursor-pointer transition-all duration-200 hover:bg-accent hover:border-accent-foreground/20 active:scale-98 ${values.blockOriginalEvent ? "border-primary/50 bg-primary/5" : ""}`}
            onClick={() =>
              update("blockOriginalEvent", !values.blockOriginalEvent)
            }
          >
            <Checkbox
              id="blockOriginalEvent"
              checked={values.blockOriginalEvent}
              onCheckedChange={(c) => update("blockOriginalEvent", !!c)}
              className="pointer-events-none"
            />
            <div className="flex flex-col gap-0.5">
              <Label className="text-sm font-medium cursor-pointer">
                Block original event
              </Label>
              <p className="text-xs text-muted-foreground">
                Prevent the trigger key from reaching other applications
              </p>
            </div>
          </div>

          <div
            className={`flex items-center gap-3 p-3 rounded-md border bg-background cursor-pointer transition-all duration-200 hover:bg-accent hover:border-accent-foreground/20 active:scale-98 ${values.ignoreInjectedKeys ? "border-primary/50 bg-primary/5" : ""}`}
            onClick={() =>
              update("ignoreInjectedKeys", !values.ignoreInjectedKeys)
            }
          >
            <Checkbox
              id="ignoreInjectedKeys"
              checked={values.ignoreInjectedKeys}
              onCheckedChange={(c) => update("ignoreInjectedKeys", !!c)}
              className="pointer-events-none"
            />
            <div className="flex flex-col gap-0.5">
              <Label className="text-sm font-medium cursor-pointer">
                Ignore injected keys
              </Label>
              <p className="text-xs text-muted-foreground">
                Ignore events generated by this macro
              </p>
            </div>
          </div>
        </div>
      </div>

      <Separator />

      {/* Enable Switch */}
      <div
        className={`flex items-center justify-between p-4 rounded-md border bg-background cursor-pointer transition-all duration-200 hover:bg-accent hover:border-accent-foreground/20 active:scale-99 ${values.enabled ? "border-primary/50 bg-primary/5" : ""}`}
          onClick={() => update("enabled", !values.enabled)}
        >
          <div className="flex flex-col gap-1">
            <Label className="text-sm font-medium cursor-pointer">
              Enable macro
            </Label>
            <p className="text-xs text-muted-foreground">
              Toggle the macro on or off
            </p>
          </div>
          <Switch
            id="enabled"
            checked={values.enabled}
            onCheckedChange={(c) => update("enabled", c)}
            className="pointer-events-none"
          />
        </div>
    </div>
  );
}
