import { Action } from "./Action";
import { Mode } from "./Mode";

export interface Macro {
    triggerKey: string;
    mode: Mode;
    actions: Action[],
    blockOriginalKey: boolean;
    ignoreInjectedKeys: boolean;
    enabled: boolean;
    collectionId: string | null;
}