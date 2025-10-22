import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface WorkspaceInfo {
  path: string;
}

export interface DatasetInfo {
  id: number;
  name: string;
  description: string;
  tags: string[];
  created_at: Date;
}

export function getWorkspaceInfo(): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("get_workspace_info");
}

export async function listDatasets(): Promise<DatasetInfo[]> {
  interface RawDatasetInfo {
    id: number;
    name: string;
    description: string;
    tags: string[];
    created_at: string;
  }

  const rawDatasets = await invoke<RawDatasetInfo[]>("list_datasets");
  return rawDatasets.map((dataset) => ({
    ...dataset,
    created_at: new Date(dataset.created_at),
  }));
}

export interface DatasetCreatedEvent {
  id: number;
  uid: string;
  name: string;
  description: string;
  tags: string[];
}

export function onDatasetCreated(
  callback: (event: DatasetCreatedEvent) => void,
) {
  return listen<DatasetCreatedEvent>("dataset-created", (event) => {
    callback(event.payload);
  });
}
