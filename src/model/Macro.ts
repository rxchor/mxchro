import { ActionConfig } from "./ActionConfig";
import { Mode } from "./Mode";

export interface Macro {
    triggerKey: string;
    mode: Mode;
    actions: ActionConfig[],
    blockOriginalKey: boolean;
    ignoreInjectedKeys: boolean;
    enabled: boolean;
}