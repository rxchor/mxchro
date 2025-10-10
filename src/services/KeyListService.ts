import { invoke } from "@tauri-apps/api/core";

const keys: string[] = await getKeysFromBackend();

async function getKeysFromBackend(): Promise<string[]> {
    console.log("getting the keys from the backend...");
    return await invoke<string[]>("get_vk_string_values");
}

export function getKeys(): string[] {
  return keys;
}

export function formattedKey(key: string): string {
  return (key.charAt(0).toUpperCase() + key.slice(1).toLowerCase()).replace(
        "_",
        " "
      );
}