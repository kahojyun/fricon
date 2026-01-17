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
  favorite: boolean;
  tags: string[];
  createdAt: string;
}

export interface DatasetInfo {
  id: number;
  name: string;
  description: string;
  favorite: boolean;
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
  /** Indices of chosen values for each filter field */
  indexFilters?: number[];
  excludeColumns?: string[];
  columns?: number[];
}

export function getWorkspaceInfo(): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("get_workspace_info");
}

export async function listDatasets(
  search?: string,
  tags?: string[],
): Promise<DatasetInfo[]> {
  const rawDatasets = await invoke<RawDatasetInfo[]>("list_datasets", {
    options: {
      search: search?.trim() || undefined,
      tags: tags && tags.length > 0 ? tags : undefined,
    },
  });
  return rawDatasets.map((dataset) => ({
    ...dataset,
    createdAt: new Date(dataset.createdAt),
  }));
}

export async function updateDatasetFavorite(
  id: number,
  favorite: boolean,
): Promise<void> {
  await invoke("update_dataset_favorite", { id, update: { favorite } });
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

export interface FilterTableRow {
  displayValues: string[];
  valueIndices: number[];
  index: number;
}

export interface ColumnUniqueValue {
  index: number;
  displayValue: string;
}

export interface FilterTableData {
  fields: string[];
  rows: FilterTableRow[];
  columnUniqueValues: Record<string, ColumnUniqueValue[]>;
}

export interface FilterTableOptions {
  excludeColumns?: string[];
}

export function getFilterTableData(
  id: number,
  options: FilterTableOptions,
): Promise<FilterTableData> {
  return invoke<FilterTableData>("get_filter_table_data", { id, options });
}
