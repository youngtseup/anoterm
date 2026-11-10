import { invoke } from "@tauri-apps/api/core";

export async function getSetting(key: string): Promise<string | null> {
  return await invoke<string | null>("get_setting", { key });
}

export async function setSetting(key: string, value: string): Promise<void> {
  await invoke("set_setting", { key, value });
}
