import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { Loader2Icon } from "lucide-react";
import { useState } from "react";

export function HomePage() {

    const [disabled, setDisabled] = useState(false);


    function handleClick() {
        setDisabled(true);
        invoke<void>('macro_test');
        setTimeout(() => setDisabled(false), 3000);
    }

    return (
        <div className="flex flex-col gap-4 items-center">
            <h1 className='font-semibold text-3xl'>Home Page</h1>
            <Button 
                disabled={disabled} 
                onClick={handleClick} 
                className="cursor-pointer"
            >
                {disabled && <Loader2Icon className="animate-spin" />}
                {disabled ? 'Processing...' : 'Click me to see magic... heh...'}
            </Button>
        </div>
    );
}