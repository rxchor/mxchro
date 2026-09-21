import { Mode } from "@/model/Mode";
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
import { Label } from "@/components/ui/label";
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
});

export type MacroFormValues = z.infer<typeof macroFormSchema>;

interface MacroFormProps {
  values: MacroFormValues;
  onChange: (values: MacroFormValues) => void;
}

export default function MacroForm({ values, onChange }: MacroFormProps) {
  const [cbKeys, setCbKeys] = useState<ComboboxData[]>([]);

  useEffect(() => {
    setCbKeys(
      getKeys().map((key) => ({
        label: formattedKey(key),
        value: key,
      }))
    );
  }, []);

  const modes = Object.values(Mode);

  function update<K extends keyof MacroFormValues>(
    key: K,
    value: MacroFormValues[K]
  ) {
    onChange({ ...values, [key]: value });
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 bg-primary-foreground p-5 rounded-md">
      <div className="flex flex-col gap-2">
        <Label>Trigger Key</Label>
        <Combobox
          value={values.triggerKey}
          onValueChange={(v) => update("triggerKey", v)}
          data={cbKeys}
          type="key"
        >
          <ComboboxTrigger placeholder="Select a key" />
          <ComboboxContent className="h-[100%]">
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
        <Label>Mode</Label>
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

      <div className="flex items-center gap-2 lg:pt-6">
        <Checkbox
          id="blockOriginalEvent"
          checked={values.blockOriginalEvent}
          onCheckedChange={(c) => update("blockOriginalEvent", !!c)}
        />
        <Label htmlFor="blockOriginalEvent">Block original event</Label>
      </div>

      <div className="flex items-center gap-2 lg:pt-6">
        <Checkbox
          id="ignoreInjectedKeys"
          checked={values.ignoreInjectedKeys}
          onCheckedChange={(c) => update("ignoreInjectedKeys", !!c)}
        />
        <Label htmlFor="ignoreInjectedKeys">Ignore injected keys</Label>
      </div>
    </div>
  );
}
