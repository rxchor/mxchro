import z from "zod";
import MacroForm, { macroFormSchema } from "./MacroForm";
import MacroActions from "./MacroActions";

export function CreateMacroPage() {
  /* SUBMIT FUNCTION */
  function onSubmit(values: z.infer<typeof macroFormSchema>) {
    console.log(values);
  }

  return (
    <div className="grid grid-cols-2 grid-rows-[auto_1fr] gap-8 w-full h-[70vh] m-20">
      <MacroForm onSubmit={onSubmit} />
      <MacroActions />
    </div>
  );
}