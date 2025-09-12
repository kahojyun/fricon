import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

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

export interface DatasetCreatedEvent {
  id: number;
  uuid: string;
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

// Plot configuration types
export interface ColumnPlotConfig {
  name: string;
  data_type: string;
  can_be_x_axis: boolean;
  can_be_y_axis: boolean;
  suggested_plot_types: string[];
  settings: Record<string, string>;
}

export interface MultiIndex {
  level_indices: number[];
  level_names: string[];
  deepest_level_col: number | null;
}

export interface DatasetPlotConfig {
  dataset_name: string;
  columns: ColumnPlotConfig[];
  settings: Record<string, string>;
  multi_index: MultiIndex | null;
}

export async function getDatasetPlotConfig(datasetId: number): Promise<DatasetPlotConfig> {
  return await invoke<DatasetPlotConfig>("get_dataset_plot_config", { datasetId });
}
