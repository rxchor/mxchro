import { Mode } from "./Mode";

export interface MacroView {
    triggerKey: string;
    mode: Mode;
    actionsCount: number;
    blockOriginalEvent: boolean;
    ignoreInjectedKeys: boolean;
    enabled: boolean;
}
