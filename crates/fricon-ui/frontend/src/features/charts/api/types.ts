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
  LiveChartDataOptions as WireLiveChartDataOptions,
  LiveChartDataResponse as WireLiveChartResponse,
  Row as FilterTableRow,
  TableData as WireFilterTableData,
  UiDatasetStatus as DatasetStatus,
} from "@/shared/lib/bindings";
import type {
  ChartModel,
  ComplexViewOption,
  XYDrawStyle,
} from "@/shared/lib/chartTypes";

export type {
  ColumnInfo,
  ColumnUniqueValue,
  DatasetStatus,
  DatasetWriteStatus,
  FilterTableOptions,
  FilterTableRow,
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

interface XYRoleOptions {
  groupByIndexColumns?: string[];
  orderByIndexColumn?: string | null;
}

interface TrendProjectionOptions {
  projection: "trend";
  series: string;
  complexViews?: ComplexViewOption[];
}

type XYProjectionOptions =
  | TrendProjectionOptions
  | {
      projection: "xy";
      xColumn: string;
      yColumn: string;
    }
  | {
      projection: "complex_xy";
      series: string;
    };

export type ChartDataOptions =
  | (BaseChartDataOptions & {
      view: "xy";
      drawStyle: XYDrawStyle;
    } & XYRoleOptions &
      XYProjectionOptions)
  | (BaseChartDataOptions & {
      view: "heatmap";
      series: string;
      xColumn?: string;
      yColumn: string;
      complexViewSingle?: ComplexViewOption;
    });

export type LiveChartDataOptions =
  | ({
      view: "xy";
      drawStyle: XYDrawStyle;
      tailCount: number;
      knownRowCount?: number | null;
    } & XYRoleOptions &
      XYProjectionOptions)
  | {
      view: "heatmap";
      series: string;
      complexViewSingle?: ComplexViewOption;
      knownRowCount?: number | null;
    };

export type LiveChartAppendOperation =
  | {
      kind: "append_points";
      seriesId: string;
      values: Float64Array;
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
  if (options.view === "heatmap") {
    return {
      view: "heatmap",
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

  return {
    view: "xy",
    drawStyle: options.drawStyle,
    ...toWireXYProjection(options),
    groupByIndexColumns: options.groupByIndexColumns ?? null,
    orderByIndexColumn: options.orderByIndexColumn ?? null,
    start: options.start ?? null,
    end: options.end ?? null,
    indexFilters: options.indexFilters ?? null,
    excludeColumns: options.excludeColumns ?? null,
  };
}

export function toWireLiveChartOptions(
  options: LiveChartDataOptions,
): WireLiveChartDataOptions {
  if (options.view === "heatmap") {
    return {
      view: "heatmap",
      series: options.series,
      complexViewSingle: options.complexViewSingle ?? null,
      knownRowCount: options.knownRowCount ?? null,
    };
  }

  return {
    view: "xy",
    drawStyle: options.drawStyle,
    tailCount: options.tailCount,
    knownRowCount: options.knownRowCount ?? null,
    groupByIndexColumns: options.groupByIndexColumns ?? null,
    orderByIndexColumn: options.orderByIndexColumn ?? null,
    ...toWireXYProjection(options),
  };
}

export function normalizeChartSnapshot(result: WireChartSnapshot): ChartModel {
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
    type: "xy",
    projection: result.projection,
    drawStyle: result.drawStyle,
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

function toWireXYProjection(options: XYProjectionOptions):
  | {
      projection: "trend";
      series: string;
      complex_views: ComplexViewOption[] | null;
    }
  | {
      projection: "xy";
      xColumn: string;
      yColumn: string;
    }
  | {
      projection: "complex_xy";
      series: string;
    } {
  switch (options.projection) {
    case "trend":
      return {
        projection: "trend",
        series: options.series,
        complex_views: options.complexViews ?? null,
      };
    case "xy":
      return {
        projection: "xy",
        xColumn: options.xColumn,
        yColumn: options.yColumn,
      };
    case "complex_xy":
      return {
        projection: "complex_xy",
        series: options.series,
      };
  }
}

function normalizeXYSeries(series: WireFlatXYSeries) {
  return {
    id: series.id,
    label: series.label,
    values: Float64Array.from(series.values),
    pointCount: series.pointCount,
  };
}

function normalizeXYZSeries(series: WireFlatXYZSeries) {
  return {
    id: series.id,
    label: series.label,
    values: Float64Array.from(series.values),
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
      values: Float64Array.from(operation.values),
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
