import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ChartOptions,
  ChartType,
  ComplexViewOption,
  ScatterMode,
} from "@/types/chart";

export interface WorkspaceInfo {
  path: string;
}

export type DatasetStatus = "Writing" | "Completed" | "Aborted";

interface RawDatasetInfo {
  id: number;
  name: string;
  description: string;
  favorite: boolean;
  tags: string[];
  status: DatasetStatus;
  createdAt: string;
}

export interface DatasetInfo {
  id: number;
  name: string;
  description: string;
  favorite: boolean;
  tags: string[];
  status: DatasetStatus;
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

export interface ChartDataOptions {
  chartType: ChartType;
  series?: string;
  xColumn?: string;
  yColumn?: string;
  scatterMode?: ScatterMode;
  scatterSeries?: string;
  scatterXColumn?: string;
  scatterYColumn?: string;
  scatterTraceXColumn?: string;
  scatterTraceYColumn?: string;
  scatterBinColumn?: string;
  complexViews?: ComplexViewOption[];
  complexViewSingle?: ComplexViewOption;
  start?: number;
  end?: number;
  /** Indices of chosen values for each filter field */
  indexFilters?: number[];
  excludeColumns?: string[];
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

export async function fetchChartData(
  id: number,
  options: ChartDataOptions,
): Promise<ChartOptions> {
  return invoke<ChartOptions>("dataset_chart_data", { id, options });
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

export interface DatasetWriteStatus {
  rowCount: number;
  isComplete: boolean;
}

export function getDatasetWriteStatus(id: number): Promise<DatasetWriteStatus> {
  return invoke<DatasetWriteStatus>("get_dataset_write_status", { id });
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
