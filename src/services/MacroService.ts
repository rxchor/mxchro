import { invoke } from "@tauri-apps/api/core";
import { Macro } from "@/model/Macro";
import { Mode } from "@/model/Mode";

export interface MacroSummary {
    triggerKey: string;
    mode: Mode;
    actionsCount: number;
    blockOriginalEvent: boolean;
    ignoreInjectedKeys: boolean;
    enabled: boolean;
}

interface ActionDto {
    type: "key" | "wait";
    key?: string;
    press?: boolean;
    duration_ms?: number;
}

interface MacroDto {
    trigger_key: string;
    mode: Mode;
    actions: ActionDto[];
    block_original_event: boolean;
    ignore_injected_keys: boolean;
    enabled: boolean;
}

interface MacroSummaryDto {
    trigger_key: string;
    mode: Mode;
    actions_count: number;
    block_original_event: boolean;
    ignore_injected_keys: boolean;
    enabled: boolean;
}

function toSnakeCase(macro: Macro): MacroDto {
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
    };
}

function fromSnakeCase(dto: MacroSummaryDto): MacroSummary {
    return {
        triggerKey: dto.trigger_key,
        mode: dto.mode,
        actionsCount: dto.actions_count,
        blockOriginalEvent: dto.block_original_event,
        ignoreInjectedKeys: dto.ignore_injected_keys,
        enabled: dto.enabled,
    };
}

function fromSnakeCaseToMacro(dto: MacroDto): Macro {
    return {
        triggerKey: dto.trigger_key,
        mode: dto.mode,
        actions: dto.actions.map(a => {
            if (a.type === "key") {
                return { type: "key" as const, key: a.key!, press: a.press! };
            } else {
                return { type: "wait" as const, duration_ms: a.duration_ms! };
            }
        }),
        blockOriginalKey: dto.block_original_event,
        ignoreInjectedKeys: dto.ignore_injected_keys,
        enabled: dto.enabled,
    };
}

export async function addMacro(macro: Macro): Promise<boolean> {
    const config = toSnakeCase(macro);
    return await invoke<boolean>("add_macro", { dto: config });
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

export async function listMacros(): Promise<MacroSummary[]> {
    const dtos = await invoke<MacroSummaryDto[]>("list_macros");
    return dtos.map(fromSnakeCase);
}

export async function getMacro(triggerKey: string): Promise<Macro | null> {
    const dto = await invoke<MacroDto | null>("get_macro", { triggerKey });
    return dto ? fromSnakeCaseToMacro(dto) : null;
}

export async function updateMacro(oldTriggerKey: string, macro: Macro): Promise<boolean> {
    const config = toSnakeCase(macro);
    return await invoke<boolean>("update_macro", { oldTriggerKey, dto: config });
}
