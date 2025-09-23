import { KeyEvent } from "./KeyEvent";
import { Mode } from "./Mode";

type Action = KeyEvent | number;
export interface Macro {
    triggerKey: string;
    mode: Mode;
    actions: Action[],
    blockOriginalKey: boolean;
}