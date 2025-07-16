import { invoke } from "@tauri-apps/api/core";

export async function getConnectionStatus(): Promise<string> {
  const status = await invoke<string>("get_connection_status");
  return status;
}

export async function setWorkspacePath(path: string): Promise<void> {
  await invoke("set_workspace_path", { path });
}
