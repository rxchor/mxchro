import { KeyEvent } from "./KeyEvent";
import { Mode } from "./Mode";

export interface Macro {
    triggerKey: string;
    mode: Mode;
    actions: KeyEvent[],
    blockOriginalKey: boolean;
    ignoreInjectedKeys: boolean;
}