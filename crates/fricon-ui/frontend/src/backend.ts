import { invoke } from "@tauri-apps/api/core";

export async function selectWorkspace(): Promise<void> {
  await invoke("select_workspace");
}

export async function getConnectionStatus(): Promise<string> {
  const status = await invoke<string>("get_connection_status");
  return status;
}
