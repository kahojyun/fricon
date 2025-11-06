import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { type Table, tableFromIPC } from "apache-arrow";

export interface WorkspaceInfo {
  path: string;
}

export interface DatasetInfo {
  id: number;
  name: string;
  description: string;
  tags: string[];
  createdAt: Date;
}

export interface ColumnInfo {
  name: string;
  isComplex: boolean;
  isTrace: boolean;
}

export interface DatasetDetail {
  columns: ColumnInfo[];
  index?: number[];
}

export interface DatasetDataOptions {
  start?: number;
  end?: number;
  columns?: number[];
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
    createdAt: string;
  }

  const rawDatasets = await invoke<RawDatasetInfo[]>("list_datasets");
  return rawDatasets.map((dataset) => ({
    ...dataset,
    createdAt: new Date(dataset.createdAt),
  }));
}

export async function fetchData(
  id: number,
  options: DatasetDataOptions,
): Promise<Table> {
  const buffer = await invoke<ArrayBuffer>("dataset_data", { id, options });
  return tableFromIPC(buffer);
}

export function datasetDetail(id: number): Promise<DatasetDetail> {
  return invoke<DatasetDetail>("dataset_detail", { id });
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
