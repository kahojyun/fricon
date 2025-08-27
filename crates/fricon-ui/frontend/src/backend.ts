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

// Chart types and interfaces
export type ColumnValue =
  | { type: "Number"; value: number }
  | { type: "String"; value: string }
  | { type: "Boolean"; value: boolean };

export type ColumnDataType = "Numeric" | "Text" | "Boolean" | "Other";

export interface ColumnInfo {
  name: string;
  data_type: ColumnDataType;
  is_index_column: boolean;
  unique_values?: ColumnValue[];
}

export interface ChartSchemaResponse {
  columns: ColumnInfo[];
  index_columns: string[];
}

export interface IndexColumnFilter {
  column: string;
  value: ColumnValue;
}

export interface ChartDataRequest {
  dataset_id: number;
  x_column: string;
  y_columns: string[];
  index_column_filters: IndexColumnFilter[];
}

export interface EChartsDataset {
  dimensions: string[];
  source: ColumnValue[][];
}

export interface EChartsSeries {
  name: string;
  type: string;
  data_group_id: number;
}

export interface EChartsDataResponse {
  dataset: EChartsDataset;
  series: EChartsSeries[];
}

// Chart API functions
export async function getChartSchema(
  datasetId: number,
): Promise<ChartSchemaResponse> {
  return await invoke<ChartSchemaResponse>("get_chart_schema", { datasetId });
}

export async function getChartData(
  request: ChartDataRequest,
): Promise<EChartsDataResponse> {
  return await invoke<EChartsDataResponse>("get_chart_data", { request });
}
