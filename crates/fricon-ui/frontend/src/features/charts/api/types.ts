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
  traceGroupIndexColumns?: string[];
  sweepIndexColumn?: string | null;
}

interface QuantityVsSweepPlotModeOptions {
  plotMode: "quantity_vs_sweep";
  quantity: string;
  complexViews?: ComplexViewOption[];
}

type XYPlotModeOptions =
  | QuantityVsSweepPlotModeOptions
  | {
      plotMode: "xy";
      xColumn: string;
      yColumn: string;
    }
  | {
      plotMode: "complex_plane";
      quantity: string;
    };

export type ChartDataOptions =
  | (BaseChartDataOptions & {
      view: "xy";
      drawStyle: XYDrawStyle;
    } & XYRoleOptions &
      XYPlotModeOptions)
  | (BaseChartDataOptions & {
      view: "heatmap";
      quantity: string;
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
      XYPlotModeOptions)
  | {
      view: "heatmap";
      quantity: string;
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
      quantity: options.quantity,
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
    ...toWireXYPlotMode(options),
    traceGroupIndexColumns: options.traceGroupIndexColumns ?? null,
    sweepIndexColumn: options.sweepIndexColumn ?? null,
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
      quantity: options.quantity,
      complexViewSingle: options.complexViewSingle ?? null,
      knownRowCount: options.knownRowCount ?? null,
    };
  }

  return {
    view: "xy",
    drawStyle: options.drawStyle,
    tailCount: options.tailCount,
    knownRowCount: options.knownRowCount ?? null,
    traceGroupIndexColumns: options.traceGroupIndexColumns ?? null,
    sweepIndexColumn: options.sweepIndexColumn ?? null,
    ...toWireXYPlotMode(options),
  };
}

export function normalizeChartSnapshot(result: WireChartSnapshot): ChartModel {
  switch (result.type) {
    case "heatmap":
      return {
        type: "heatmap",
        xName: result.xName,
        yName: result.yName,
        series: result.series.map(normalizeXYZSeries),
      };
    case "xy":
      return {
        type: "xy",
        plotMode: result.plotMode,
        drawStyle: result.drawStyle,
        xName: result.xName,
        yName: result.yName,
        series: result.series.map(normalizeXYSeries),
      };
    default:
      return assertNever(
        result,
        `Unknown chart snapshot type: ${String((result as { type?: unknown }).type)}`,
      );
  }
}

export function normalizeLiveChartUpdate(
  result: WireLiveChartResponse,
): LiveChartUpdate {
  switch (result.mode) {
    case "reset":
      return {
        mode: "reset",
        rowCount: result.row_count,
        snapshot: normalizeChartSnapshot(result.snapshot),
      };
    case "append":
      return {
        mode: "append",
        rowCount: result.row_count,
        ops: result.ops.map(normalizeLiveChartAppendOperation),
      };
    default:
      return assertNever(
        result,
        `Unknown live chart update mode: ${String((result as { mode?: unknown }).mode)}`,
      );
  }
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

function toWireXYPlotMode(options: XYPlotModeOptions):
  | {
      plotMode: "quantity_vs_sweep";
      quantity: string;
      complex_views: ComplexViewOption[] | null;
    }
  | {
      plotMode: "xy";
      xColumn: string;
      yColumn: string;
    }
  | {
      plotMode: "complex_plane";
      quantity: string;
    } {
  switch (options.plotMode) {
    case "quantity_vs_sweep":
      return {
        plotMode: "quantity_vs_sweep",
        quantity: options.quantity,
        complex_views: options.complexViews ?? null,
      };
    case "xy":
      return {
        plotMode: "xy",
        xColumn: options.xColumn,
        yColumn: options.yColumn,
      };
    case "complex_plane":
      return {
        plotMode: "complex_plane",
        quantity: options.quantity,
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
  switch (series.shape) {
    case "xy":
      return {
        shape: "xy" as const,
        series: normalizeXYSeries(series),
      };
    case "xyz":
      return {
        shape: "xyz" as const,
        series: normalizeXYZSeries(series),
      };
    default:
      return assertNever(
        series,
        `Unknown flat series shape: ${String((series as { shape?: unknown }).shape)}`,
      );
  }
}

function normalizeLiveChartAppendOperation(
  operation: WireLiveChartAppendOperation,
): LiveChartAppendOperation {
  switch (operation.kind) {
    case "append_points":
      return {
        kind: "append_points",
        seriesId: operation.series_id,
        values: Float64Array.from(operation.values),
        pointCount: operation.point_count,
      };
    case "append_series":
      return {
        kind: "append_series",
        series: normalizeFlatSeries(operation.series),
      };
    default:
      return assertNever(
        operation,
        `Unknown live chart operation kind: ${String((operation as { kind?: unknown }).kind)}`,
      );
  }
}

function assertNever(_value: never, message: string): never {
  throw new Error(message);
}
