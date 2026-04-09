import type {
  ChartSnapshot as WireChartSnapshot,
  ColumnUniqueValue,
  ColumnInfo,
  DatasetChartDataOptions as WireChartDataOptions,
  DatasetWriteStatus,
  FilterTableOptions,
  FlatSeries as WireFlatSeries,
  FlatXYSeries as WireFlatXYSeries,
  FlatXYZSeries as WireFlatXYZSeries,
  LiveChartAppendOperation as WireLiveChartAppendOperation,
  LiveChartDataResponse as WireLiveChartResponse,
  LiveChartDataOptions as WireLiveChartDataOptions,
  Row as FilterTableRow,
  ScatterModeOptions as WireScatterModeOptions,
  TableData as WireFilterTableData,
  UiDatasetStatus as DatasetStatus,
} from "@/shared/lib/bindings";
import type { ChartModel, ComplexViewOption } from "@/shared/lib/chartTypes";

export type {
  ColumnInfo,
  ColumnUniqueValue,
  DatasetStatus,
  DatasetWriteStatus,
  FilterTableOptions,
  FilterTableRow,
  WireLiveChartDataOptions as LiveChartDataOptions,
};

export interface DatasetDetail {
  status: DatasetStatus;
  payloadAvailable: boolean;
  columns: ColumnInfo[];
}

export type ChartViewerAvailability = "loading" | "available" | "tombstone";

export interface FilterTableData {
  fields: string[];
  rows: FilterTableRow[];
  columnUniqueValues: Record<string, ColumnUniqueValue[]>;
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

export type ScatterModeOptions = WireScatterModeOptions;

export type LiveChartAppendOperation =
  | {
      kind: "append_points";
      seriesId: string;
      values: Float32Array;
      pointCount: number;
    }
  | {
      kind: "append_series";
      series:
        | {
            shape: "xy";
            series: import("@/shared/lib/chartTypes").ChartSeries;
          }
        | {
            shape: "xyz";
            series: import("@/shared/lib/chartTypes").HeatmapSeries;
          };
    }
  | {
      kind: "append_heatmap_categories";
      xCategories?: number[];
      yCategories?: number[];
    };

export type LiveChartUpdate =
  | {
      mode: "reset";
      rowCount: number;
      snapshot: ChartModel;
    }
  | {
      mode: "append";
      rowCount: number;
      ops: LiveChartAppendOperation[];
    };

export function toWireChartOptions(
  options: ChartDataOptions,
): WireChartDataOptions {
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

export function normalizeChartSnapshot(result: WireChartSnapshot): ChartModel {
  if (result.type === "line") {
    return {
      type: "line",
      xName: result.xName,
      series: result.series.map(normalizeXYSeries),
    };
  }

  if (result.type === "heatmap") {
    return {
      type: "heatmap",
      xName: result.xName,
      yName: result.yName,
      xCategories: result.xCategories,
      yCategories: result.yCategories,
      series: result.series.map(normalizeXYZSeries),
    };
  }

  return {
    type: "scatter",
    xName: result.xName,
    yName: result.yName,
    series: result.series.map(normalizeXYSeries),
  };
}

export function normalizeLiveChartUpdate(
  result: WireLiveChartResponse,
): LiveChartUpdate {
  if (result.mode === "reset") {
    return {
      mode: "reset",
      rowCount: result.row_count,
      snapshot: normalizeChartSnapshot(result.snapshot),
    };
  }

  return {
    mode: "append",
    rowCount: result.row_count,
    ops: result.ops.map(normalizeLiveChartAppendOperation),
  };
}

export function normalizeFilterTableData(
  result: WireFilterTableData,
): FilterTableData {
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

function normalizeXYSeries(series: WireFlatXYSeries) {
  return {
    id: series.id,
    label: series.label,
    values: Float32Array.from(series.values),
    pointCount: series.pointCount,
  };
}

function normalizeXYZSeries(series: WireFlatXYZSeries) {
  return {
    id: series.id,
    label: series.label,
    values: Float32Array.from(series.values),
    pointCount: series.pointCount,
  };
}

function normalizeFlatSeries(series: WireFlatSeries) {
  if (series.shape === "xy") {
    return {
      shape: "xy" as const,
      series: normalizeXYSeries(series),
    };
  }

  return {
    shape: "xyz" as const,
    series: normalizeXYZSeries(series),
  };
}

function normalizeLiveChartAppendOperation(
  operation: WireLiveChartAppendOperation,
): LiveChartAppendOperation {
  if (operation.kind === "append_points") {
    return {
      kind: "append_points",
      seriesId: operation.series_id,
      values: Float32Array.from(operation.values),
      pointCount: operation.point_count,
    };
  }

  if (operation.kind === "append_series") {
    return {
      kind: "append_series",
      series: normalizeFlatSeries(operation.series),
    };
  }

  return {
    kind: "append_heatmap_categories",
    xCategories: operation.x_categories ?? undefined,
    yCategories: operation.y_categories ?? undefined,
  };
}
