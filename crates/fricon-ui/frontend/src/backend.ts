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
  created_at: Date;
}

interface RawDatasetInfo {
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
  const rawDatasets = await invoke<RawDatasetInfo[]>("list_datasets");
  return rawDatasets.map((dataset) => ({
    ...dataset,
    created_at: new Date(dataset.created_at),
  }));
}
