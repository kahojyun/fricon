import {
  commands,
  events,
  type ColumnInfo,
  type ColumnUniqueValue,
  type DatasetChartDataOptions as WireChartDataOptions,
  type DataResponse as WireChartResponse,
  type DatasetDetail as WireDatasetDetail,
  type DatasetFavoriteUpdate,
  type DatasetInfo as WireDatasetInfo,
  type DatasetInfoUpdate as WireDatasetInfoUpdate,
  type DatasetListOptions as WireDatasetListOptions,
  type DatasetWriteStatus,
  type Error as WireError,
  type FilterTableOptions as WireFilterTableOptions,
  type Row as FilterTableRow,
  type TableData as WireFilterTableData,
  type UiDatasetSortBy as DatasetListSortBy,
  type UiDatasetStatus as DatasetStatus,
  type UiSortDirection as DatasetListSortDir,
  type WorkspaceInfo,
} from "@/lib/bindings";
import type { ChartOptions, ComplexViewOption } from "@/lib/chartTypes";

function unwrapResult<T>(
  result: { status: "ok"; data: T } | { status: "error"; error: WireError },
): T {
  if (result.status === "ok") {
    return result.data;
  }
  throw new Error(result.error.message);
}

function toDate(value: string): Date {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    throw new Error(`Invalid date value from backend: ${value}`);
  }
  return date;
}

function toWireChartOptions(options: ChartDataOptions): WireChartDataOptions {
  if (options.chartType === "line") {
    return {
      chartType: "line",
      series: options.series,
      xColumn: options.xColumn ?? null,
      complexViews: options.complexViews ?? null,
      start: options.start ?? null,
      end: options.end ?? null,
      indexFilters: options.indexFilters ?? null,
      excludeColumns: options.excludeColumns ?? null,
    };
  }

  if (options.chartType === "heatmap") {
    return {
      chartType: "heatmap",
      series: options.series,
      xColumn: options.xColumn ?? null,
      yColumn: options.yColumn,
      complexViewSingle: options.complexViewSingle ?? null,
      start: options.start ?? null,
      end: options.end ?? null,
      indexFilters: options.indexFilters ?? null,
      excludeColumns: options.excludeColumns ?? null,
    };
  }

  const scatter = (() => {
    if (options.scatter.mode === "complex") {
      return {
        mode: "complex" as const,
        series: options.scatter.series,
      };
    }
    if (options.scatter.mode === "trace_xy") {
      return {
        mode: "trace_xy" as const,
        traceXColumn: options.scatter.traceXColumn,
        traceYColumn: options.scatter.traceYColumn,
      };
    }
    return {
      mode: "xy" as const,
      xColumn: options.scatter.xColumn,
      yColumn: options.scatter.yColumn,
      binColumn: options.scatter.binColumn ?? null,
    };
  })();

  return {
    chartType: "scatter",
    scatter,
    start: options.start ?? null,
    end: options.end ?? null,
    indexFilters: options.indexFilters ?? null,
    excludeColumns: options.excludeColumns ?? null,
  };
}

function normalizeDataset(dataset: WireDatasetInfo): DatasetInfo {
  return {
    ...dataset,
    createdAt: toDate(dataset.createdAt),
  };
}

function normalizeChartOptions(result: WireChartResponse): ChartOptions {
  if (result.type === "line") {
    return {
      type: "line",
      xName: result.xName,
      series: result.series,
    };
  }

  if (result.yName == null) {
    throw new Error(
      `Missing yName for chart type '${result.type}' in backend response`,
    );
  }

  return {
    type: result.type,
    xName: result.xName,
    yName: result.yName,
    series: result.series,
  };
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

export interface DatasetDetail extends DatasetInfo {
  columns: ColumnInfo[];
}

export { type WorkspaceInfo, type DatasetStatus, type ColumnInfo };

export const DATASET_PAGE_SIZE = 200;

export type { DatasetListSortBy, DatasetListSortDir };

export interface ListDatasetsOptions {
  search?: string;
  tags?: string[];
  favoriteOnly?: boolean;
  statuses?: DatasetStatus[];
  sortBy?: DatasetListSortBy;
  sortDir?: DatasetListSortDir;
  limit?: number;
  offset?: number;
}

interface BaseChartDataOptions {
  start?: number;
  end?: number;
  indexFilters?: number[];
  excludeColumns?: string[];
}

export type ChartDataOptions =
  | (BaseChartDataOptions & {
      chartType: "line";
      series: string;
      xColumn?: string;
      complexViews?: ComplexViewOption[];
    })
  | (BaseChartDataOptions & {
      chartType: "heatmap";
      series: string;
      xColumn?: string;
      yColumn: string;
      complexViewSingle?: ComplexViewOption;
    })
  | (BaseChartDataOptions & {
      chartType: "scatter";
      scatter: ScatterModeOptions;
    });

export type ScatterModeOptions =
  | {
      mode: "complex";
      series: string;
    }
  | {
      mode: "trace_xy";
      traceXColumn: string;
      traceYColumn: string;
    }
  | {
      mode: "xy";
      xColumn: string;
      yColumn: string;
      binColumn?: string;
    };

export async function getWorkspaceInfo(): Promise<WorkspaceInfo> {
  return unwrapResult(await commands.getWorkspaceInfo());
}

export async function listDatasets(
  options: ListDatasetsOptions = {},
): Promise<DatasetInfo[]> {
  const {
    search,
    tags,
    favoriteOnly,
    statuses,
    sortBy,
    sortDir,
    limit,
    offset,
  } = options;
  const wireOptions: WireDatasetListOptions = {
    search: search?.trim() ?? null,
    tags: tags && tags.length > 0 ? tags : null,
    favoriteOnly: favoriteOnly ? true : null,
    statuses: statuses && statuses.length > 0 ? statuses : null,
    sortBy: sortBy ?? null,
    sortDir: sortDir ?? null,
    limit: limit ?? null,
    offset: offset ?? null,
  };
  const datasets = unwrapResult(await commands.listDatasets(wireOptions));
  return datasets.map(normalizeDataset);
}

export async function listDatasetTags(): Promise<string[]> {
  return unwrapResult(await commands.listDatasetTags());
}

export async function updateDatasetFavorite(
  id: number,
  favorite: boolean,
): Promise<void> {
  const update: DatasetFavoriteUpdate = { favorite };
  unwrapResult(await commands.updateDatasetFavorite(id, update));
}

export interface DatasetInfoUpdate {
  name?: string;
  description?: string;
  favorite?: boolean;
  tags?: string[];
}

export async function updateDatasetInfo(
  id: number,
  update: DatasetInfoUpdate,
): Promise<void> {
  const wireUpdate: WireDatasetInfoUpdate = {
    name: update.name ?? null,
    description: update.description ?? null,
    favorite: update.favorite ?? null,
    tags: update.tags ?? null,
  };
  unwrapResult(await commands.updateDatasetInfo(id, wireUpdate));
}

export async function fetchChartData(
  id: number,
  options: ChartDataOptions,
): Promise<ChartOptions> {
  const result: WireChartResponse = unwrapResult(
    await commands.datasetChartData(id, toWireChartOptions(options)),
  );
  return normalizeChartOptions(result);
}

export async function getDatasetDetail(id: number): Promise<DatasetDetail> {
  const rawDetail: WireDatasetDetail = unwrapResult(
    await commands.datasetDetail(id),
  );
  return {
    ...rawDetail,
    createdAt: toDate(rawDetail.createdAt),
  };
}

export function onDatasetCreated(callback: (event: DatasetInfo) => void) {
  return events.datasetCreated.listen((event) => {
    callback(normalizeDataset(event.payload));
  });
}

export function onDatasetUpdated(callback: (event: DatasetInfo) => void) {
  return events.datasetUpdated.listen((event) => {
    callback(normalizeDataset(event.payload));
  });
}

export { type DatasetWriteStatus };

export async function getDatasetWriteStatus(
  id: number,
): Promise<DatasetWriteStatus> {
  return unwrapResult(await commands.getDatasetWriteStatus(id));
}

export interface FilterTableOptions {
  excludeColumns?: string[];
}

export interface FilterTableData {
  fields: string[];
  rows: FilterTableRow[];
  columnUniqueValues: Record<string, ColumnUniqueValue[]>;
}

export { type ColumnUniqueValue, type FilterTableRow };

export async function getFilterTableData(
  id: number,
  options: FilterTableOptions,
): Promise<FilterTableData> {
  const wireOptions: WireFilterTableOptions = {
    excludeColumns: options.excludeColumns ?? null,
  };
  const result: WireFilterTableData = unwrapResult(
    await commands.getFilterTableData(id, wireOptions),
  );
  const columnUniqueValues = Object.fromEntries(
    result.fields.map((field) => [
      field,
      result.columnUniqueValues[field] ?? [],
    ]),
  );
  return {
    fields: result.fields,
    rows: result.rows,
    columnUniqueValues,
  };
}
