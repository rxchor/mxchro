import React, { useState, useEffect } from "react";
import { Input } from "./ui/input";
import { cn } from "@/lib/utils";

interface RecordKeyProps {
    value?: string;
    onChange?: (key: string) => void;
}

export default function RecordKey({
    value,
    onChange,
    className,
    ...props
}: RecordKeyProps & React.ComponentProps<"input">) {
    const [recording, setRecording] = useState(false);
    const [recordedKey, setRecordedKey] = useState<string | null>(null);

    useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent) => {
            if (!recording) return;

            event.preventDefault();
            let key = event.key.length == 1 ? event.key.toUpperCase() : event.key;
            if (key === " ") key = "Space"
            setRecordedKey(key);
            setRecording(false);
            if (onChange) {
                onChange(key);
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, [recording, onChange]);

  return (
    <Input 
        {...props} 
        className={cn(className)} 
        readOnly={true} 
        value={!recording && recordedKey ? recordedKey : ''} 
        placeholder={recording ? "Recording..." : (props.placeholder ? props.placeholder : "Click me to record")} 
        onClick={() => { 
            setRecording(true); 
            setRecordedKey(null);
        }}
    />
  )
};
