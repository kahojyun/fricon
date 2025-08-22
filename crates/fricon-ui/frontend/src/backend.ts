import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceInfo {
  path: string;
  is_ready: boolean;
}

export interface ServerStatus {
  is_running: boolean;
  ipc_path: string;
}

export interface DatasetInfo {
  id: number;
  name: string;
  description: string;
  tags: string[];
  created_at: string;
}

export async function getWorkspaceInfo(): Promise<WorkspaceInfo> {
  return await invoke<WorkspaceInfo>("get_workspace_info");
}

export async function getServerStatus(): Promise<ServerStatus> {
  return await invoke<ServerStatus>("get_server_status");
}

export async function listDatasets(): Promise<DatasetInfo[]> {
  return await invoke<DatasetInfo[]>("list_datasets");
}

export async function createDataset(
  name: string,
  description: string,
  tags: string[],
  indexColumns: string[]
): Promise<number> {
  return await invoke<number>("create_dataset", {
    name,
    description,
    tags,
    indexColumns,
  });
}

export async function shutdownServer(): Promise<void> {
  await invoke("shutdown_server");
}
