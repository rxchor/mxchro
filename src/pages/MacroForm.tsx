import { Mode } from "@/model/Mode";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
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

const formSchema = z.object({
  triggerKey: z
    .string()
    .min(1, { message: "The macro must have a trigger key" }),
  mode: z.string().min(1, { message: "Please select a mode" }),
  blockOriginalEvent: z.boolean(),
  ignoreInjectedKeys: z.boolean(),
});

export default function MacroForm() {
  const [cbKeys, setCbKeys] = useState<ComboboxData[]>([]);

  useEffect(() => {
    async function fetchKeys() {
      console.log("getting the keys from the backend...");
      const stringList = await invoke<string[]>("get_vk_string_values");
      setCbKeys(
        stringList.map((key) => {
          return {
            label: getLabel(key),
            value: key,
          };
        })
      );
    }

    function getLabel(key: string) {
      return (key.charAt(0).toUpperCase() + key.slice(1).toLowerCase()).replace(
        "_",
        " "
      );
    }

    fetchKeys();
  }, []);

  /* FORM RELATED STUFF */
  const modes = Object.values(Mode);
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      triggerKey: "",
      mode: "",
      blockOriginalEvent: true,
      ignoreInjectedKeys: true,
    },
  });

  /* SUBMIT FUNCTION */
  function onSubmit(values: z.infer<typeof formSchema>) {
    console.log(values);
  }

  // class names for the divs in case it is needed:
  // className="col-span-2 grid grid-cols-1 gap-2"

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className="grid grid-cols-2 items-center gap-8 bg-primary-foreground p-5 rounded-md"
      >
        <FormField
          control={form.control}
          name="triggerKey"
          render={({ field, fieldState }) => (
            <FormItem>
              <FormLabel>Trigger Key</FormLabel>
              <FormControl>
                <Combobox
                  value={field.value}
                  onValueChange={field.onChange}
                  data={cbKeys}
                  type="key"
                >
                  <ComboboxTrigger
                    placeholder="Select a key"
                    aria-invalid={fieldState.invalid}
                  />
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
              </FormControl>
              <FormDescription>Key that triggers the action</FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="mode"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Mode</FormLabel>
              <Select onValueChange={field.onChange} defaultValue={field.value}>
                <FormControl>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select a mode" />
                  </SelectTrigger>
                </FormControl>
                <SelectContent>
                  {modes.map((mode) => (
                    <SelectItem key={mode} value={mode.toString()}>
                      {mode.charAt(0).toUpperCase() +
                        mode.slice(1).toLowerCase()}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <FormDescription>
                The way the macro is going to execute
              </FormDescription>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="blockOriginalEvent"
          render={({ field }) => (
            <FormItem className="flex flex-row items-center gap-2">
              <FormControl>
                <Checkbox
                  checked={field.value}
                  onCheckedChange={field.onChange}
                />
              </FormControl>
              <FormLabel>Block original event</FormLabel>
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="ignoreInjectedKeys"
          render={({ field }) => (
            <FormItem className="flex flex-row items-center gap-2">
              <FormControl>
                <Checkbox
                  checked={field.value}
                  onCheckedChange={field.onChange}
                />
              </FormControl>
              <FormLabel>Ignore injected keys</FormLabel>
            </FormItem>
          )}
        />
        <div className="col-span-2 flex justify-center w-full">
          <Button className="p-4 cursor-pointer" type="submit">
            Submit
          </Button>
        </div>
      </form>
    </Form>
  );
}
