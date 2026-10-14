import { invoke } from "@tauri-apps/api/core";
import { Macro } from "@/model/Macro";
import { Mode } from "@/model/Mode";
import { MacroView } from "@/model/MacroView";

interface MacroResponse {
    trigger_key: string;
    mode: Mode;
    actions: ActionResponse[];
    block_original_event: boolean;
    ignore_injected_keys: boolean;
    enabled: boolean;
    collection_id: string | null;
    toggle_enabled_hotkey: string | null;
}

interface MacroViewResponse {
    trigger_key: string;
    mode: Mode;
    actions_count: number;
    block_original_event: boolean;
    ignore_injected_keys: boolean;
    enabled: boolean;
    collection_id: string | null;
    toggle_enabled_hotkey: string | null;
}

interface ActionResponse {
    type: "key" | "wait";
    key?: string;
    press?: boolean;
    duration_ms?: number;
}

function toSnakeCase(macro: Macro): MacroResponse {
    return {
        trigger_key: macro.triggerKey,
        mode: macro.mode,
        actions: macro.actions.map(a => {
            if (a.type === "key") {
                return {
                    type: "key" as const,
                    key: a.key,
                    press: a.press,
                };
            } else {
                return {
                    type: "wait" as const,
                    duration_ms: a.duration_ms,
                };
            }
        }),
        block_original_event: macro.blockOriginalKey,
        ignore_injected_keys: macro.ignoreInjectedKeys,
        enabled: macro.enabled,
        collection_id: macro.collectionId,
        toggle_enabled_hotkey: macro.toggleEnabledHotkey,
    };
}

function fromSnakeCase(response: MacroViewResponse): MacroView {
    return {
        triggerKey: response.trigger_key,
        mode: response.mode,
        actionsCount: response.actions_count,
        blockOriginalEvent: response.block_original_event,
        ignoreInjectedKeys: response.ignore_injected_keys,
        enabled: response.enabled,
        collectionId: response.collection_id,
        toggleEnabledHotkey: response.toggle_enabled_hotkey,
    };
}

function fromSnakeCaseToMacro(response: MacroResponse): Macro {
    return {
        triggerKey: response.trigger_key,
        mode: response.mode,
        actions: response.actions.map(a => {
            if (a.type === "key") {
                return { type: "key" as const, key: a.key!, press: a.press! };
            } else {
                return { type: "wait" as const, duration_ms: a.duration_ms! };
            }
        }),
        blockOriginalKey: response.block_original_event,
        ignoreInjectedKeys: response.ignore_injected_keys,
        enabled: response.enabled,
        collectionId: response.collection_id,
        toggleEnabledHotkey: response.toggle_enabled_hotkey,
    };
}

export async function addMacro(macro: Macro): Promise<boolean> {
    const request = toSnakeCase(macro);
    return await invoke<boolean>("add_macro", { dto: request });
}

export async function deleteMacro(triggerKey: string): Promise<boolean> {
    return await invoke<boolean>("delete_macro", { triggerKey });
}

export async function enableMacro(triggerKey: string): Promise<boolean> {
    return await invoke<boolean>("enable_macro", { triggerKey });
}

export async function disableMacro(triggerKey: string): Promise<boolean> {
    return await invoke<boolean>("disable_macro", { triggerKey });
}

export async function listMacros(): Promise<MacroView[]> {
    const responses = await invoke<MacroViewResponse[]>("list_macros");
    return responses.map(fromSnakeCase);
}

export async function getMacro(triggerKey: string): Promise<Macro | null> {
    const response = await invoke<MacroResponse | null>("get_macro", { triggerKey });
    return response ? fromSnakeCaseToMacro(response) : null;
}

export async function updateMacro(oldTriggerKey: string, macro: Macro): Promise<boolean> {
    const request = toSnakeCase(macro);
    return await invoke<boolean>("update_macro", { oldTriggerKey, dto: request });
}

export async function setMacroToggleHotkey(triggerKey: string, hotkey: string | null): Promise<boolean> {
    return await invoke<boolean>("set_macro_toggle_hotkey", { triggerKey, hotkey });
}

export async function wipeData(): Promise<void> {
    return await invoke<void>("wipe_data");
}
