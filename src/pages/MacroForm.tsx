import { Input } from "@/components/ui/input";
import { Label } from "@radix-ui/react-label";

function MacroForm() {    

    return (
        <form className="grid grid-cols-2 items-center gap-4 bg-primary-foreground p-5 rounded-md">
            <div className="col-span-2">
                <Label htmlFor="holaText">hola</Label>
                <Input onChange={ (event) => console.log('event', event)} id="holaText" type="text" placeholder="hola" />
            </div>
        </form>
    );
}

export default MacroForm;