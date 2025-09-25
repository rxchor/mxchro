import { Mode } from "@/model/Mode"
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

import { zodResolver } from "@hookform/resolvers/zod"
import { useForm } from "react-hook-form";
import { z } from "zod"
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { Button } from "@/components/ui/button";

const formSchema = z.object({
    triggerKey: z.string().min(1, {message: "The macro must have a trigger key"}),
    mode: z.string().min(1, {message: "Please select a mode"})
});

export default function MacroForm() {    
    
    const modes = Object.values(Mode);

    const form = useForm<z.infer<typeof formSchema>>({
        resolver: zodResolver(formSchema),
        defaultValues: {
            triggerKey: "",
            mode: "",
        }
    });

    function onSubmit(values: z.infer<typeof formSchema>) {

        console.log(values);
    }
    
    // class names for the divs in case it is needed:
    // className="col-span-2 grid grid-cols-1 gap-2"

    return (
        <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="grid grid-cols-2 items-center gap-8 bg-primary-foreground p-5 rounded-md">
                <FormField
                    control={form.control}
                    name="triggerKey"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>Trigger Key</FormLabel>
                            <FormControl>
                                <Input {...field} placeholder="Insert the trigger key" />
                            </FormControl>
                            <FormDescription>
                                Key that triggers the given action
                            </FormDescription>
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
                                        {modes.map((mode) => (<SelectItem key={mode} value={mode.toString()}>{mode.charAt(0).toUpperCase() + mode.slice(1).toLowerCase()}</SelectItem>))}
                                    </SelectContent>
                                </Select>
                            <FormDescription>
                                The way the macro is going to execute
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />
                <div className="col-span-2 flex justify-center w-full">
                    <Button className="p-4 cursor-pointer" type="submit">Submit</Button>
                </div>
            </form>
        </Form>
    );
}

