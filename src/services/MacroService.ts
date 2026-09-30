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

interface MacroDto {
    trigger_key: string;
    mode: Mode;
    actions: Array<{ key?: string; sleep?: number; hold?: number }>;
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
        actions: macro.actions.map(a => ({
            key: a.key,
            sleep: a.sleep,
            hold: a.hold,
        })),
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

export async function addMacro(macro: Macro): Promise<boolean> {
    const dto = toSnakeCase(macro);
    return await invoke<boolean>("add_macro", { dto });
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

export async function startHook(): Promise<void> {
    return await invoke<void>("start_hook");
}
