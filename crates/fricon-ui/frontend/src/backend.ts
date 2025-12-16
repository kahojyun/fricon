import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { type Table, tableFromIPC } from "apache-arrow";

export interface WorkspaceInfo {
  path: string;
}

interface RawDatasetInfo {
  id: number;
  name: string;
  description: string;
  tags: string[];
  createdAt: string;
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
  isIndex: boolean;
}

export interface DatasetDetail {
  columns: ColumnInfo[];
}

export interface DatasetDataOptions {
  start?: number;
  end?: number;
  /** Single row arrow table encoded with BASE64 */
  indexFilters?: string;
  columns?: number[];
}

export function getWorkspaceInfo(): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("get_workspace_info");
}

export async function listDatasets(): Promise<DatasetInfo[]> {
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

export function getDatasetDetail(id: number): Promise<DatasetDetail> {
  return invoke<DatasetDetail>("dataset_detail", { id });
}

export function onDatasetCreated(callback: (event: DatasetInfo) => void) {
  return listen<RawDatasetInfo>("dataset-created", (event) => {
    callback({
      ...event.payload,
      createdAt: new Date(event.payload.createdAt),
    });
  });
}

export interface DatasetWriteProgress {
  rowCount: number;
}

export async function subscribeDatasetUpdate(
  id: number,
  callback: (e: DatasetWriteProgress) => unknown,
) {
  const onUpdate = new Channel<DatasetWriteProgress>();
  onUpdate.onmessage = callback;
  await invoke("subscribe_dataset_update", { id, onUpdate });
  return async () => {
    await invoke("unsubscribe_dataset_update", { channelId: onUpdate.id });
  };
}
