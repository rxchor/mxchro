import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Keyboard, X } from "lucide-react";
import { getKeys, formattedKey } from "@/services/KeyListService";

interface KeyInputProps {
  value: string | null;
  onChange: (key: string | null) => void;
  label?: string;
  placeholder?: string;
}

// Mapeo de teclas del browser al formato del backend
function mapBrowserKeyToBackend(key: string, keyCode: number): string | null {
  // Teclas especiales
  const specialKeys: Record<string, string> = {
    ' ': 'Space',
    'Enter': 'Enter',
    'Escape': 'Escape',
    'Tab': 'Tab',
    'Backspace': 'Backspace',
    'Delete': 'Delete',
    'Insert': 'Insert',
    'Home': 'Home',
    'End': 'End',
    'PageUp': 'PageUp',
    'PageDown': 'PageDown',
    'ArrowUp': 'Up',
    'ArrowDown': 'Down',
    'ArrowLeft': 'Left',
    'ArrowRight': 'Right',
    'CapsLock': 'CapsLock',
    'NumLock': 'NumLock',
    'ScrollLock': 'ScrollLock',
    'Pause': 'Pause',
    'PrintScreen': 'PrintScreen',
  };

  // Función F1-F24
  if (key.match(/^F\d+$/)) {
    return key;
  }

  // Teclas especiales mapeadas
  if (specialKeys[key]) {
    return specialKeys[key];
  }

  // Numpad
  if (keyCode >= 96 && keyCode <= 105) {
    return `numpad${key}`;
  }

  // Numpad special keys
  const numpadKeys: Record<number, string> = {
    106: 'numpad_mul',
    107: 'numpad_add',
    109: 'numpad_sub',
    110: 'numpad_decimal',
    111: 'numpad_div',
  };
  if (numpadKeys[keyCode]) {
    return numpadKeys[keyCode];
  }

  // Letras A-Z (una sola carácter)
  if (key.length === 1 && key.match(/[a-zA-Z]/)) {
    return key.toUpperCase();
  }

  // Números 0-9
  if (key.length === 1 && key.match(/[0-9]/)) {
    return key;
  }

  // Símbolos comunes
  if (key.length === 1) {
    return key;
  }

  return null;
}

export function KeyInput({ value, onChange, label, placeholder = "Press a key..." }: KeyInputProps) {
  const [isListening, setIsListening] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const availableKeys = getKeys();

  useEffect(() => {
    if (!isListening) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      
      // Mapear la tecla del browser al formato del backend
      const mappedKey = mapBrowserKeyToBackend(e.key, e.keyCode);
      
      if (mappedKey && availableKeys.includes(mappedKey)) {
        onChange(mappedKey);
        setIsListening(false);
      } else {
        // Si no está en la lista, mostrar feedback pero no cambiar
        console.warn('Key not in available keys:', mappedKey, 'Browser key:', e.key);
      }
    };

    const handleBlur = () => {
      setIsListening(false);
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("blur", handleBlur);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("blur", handleBlur);
    };
  }, [isListening, onChange, availableKeys]);

  const handleClear = (e: React.MouseEvent) => {
    e.stopPropagation();
    onChange(null);
  };

  return (
    <div className="flex flex-col gap-2">
      {label && <label className="text-sm font-medium">{label}</label>}
      <div className="flex gap-2">
        <Button
          ref={buttonRef}
          variant={isListening ? "default" : "outline"}
          className={`flex-1 justify-start ${isListening ? "animate-pulse" : ""}`}
          onClick={() => setIsListening(!isListening)}
        >
          <Keyboard className="size-4 mr-2" />
          {isListening ? (
            <span className="text-muted-foreground">{placeholder}</span>
          ) : value ? (
            <span>{formattedKey(value)}</span>
          ) : (
            <span className="text-muted-foreground">Not set</span>
          )}
        </Button>
        {value && (
          <Button
            variant="ghost"
            size="icon"
            onClick={handleClear}
            className="size-9"
          >
            <X className="size-4" />
          </Button>
        )}
      </div>
    </div>
  );
}
