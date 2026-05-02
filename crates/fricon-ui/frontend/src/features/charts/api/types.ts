import type {
  ColumnUniqueValue,
  DatasetChartDataOptions as WireChartDataOptions,
  DatasetWriteStatus,
  FilterTableOptions,
  LiveChartDataOptions as WireLiveChartDataOptions,
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
  ColumnUniqueValue,
  DatasetStatus,
  DatasetWriteStatus,
  FilterTableOptions,
  FilterTableRow,
};

export interface ColumnInfo {
  name: string;
  label?: string | null;
  isComplex: boolean;
  isTrace: boolean;
  isIndex: boolean;
  hiddenByDefault?: boolean;
  isChartAxisCandidate?: boolean;
}

export type ChartSemanticAxisKind = "logical_index" | "column";

export interface ChartSemanticColumn {
  id: string;
  name: string;
  label: string | null;
  isComplex: boolean;
  isTrace: boolean;
  hiddenByDefault: boolean;
}

export interface ChartSemanticAxis {
  id: string;
  name: string;
  label: string | null;
  kind: ChartSemanticAxisKind;
  numeric: boolean;
  isCompatibility: boolean;
  physicalColumn: string | null;
}

export interface ChartSemantics {
  source: "manifest" | "compatibility_inference";
  duplicatePolicy:
    | "latest_by_record_id"
    | "compatibility_row_order_placeholder";
  indexRealization: "none" | "implicit" | "sidecar";
  axes: ChartSemanticAxis[];
  valueColumns: ChartSemanticColumn[];
  chartAxisCandidates: ChartSemanticAxis[];
}

export interface DatasetDetail {
  status: DatasetStatus;
  payloadAvailable: boolean;
  columns: ColumnInfo[];
  chartSemantics?: ChartSemantics | null;
}

export type ChartViewerAvailability = "loading" | "available" | "tombstone";

export interface FilterTableData {
  fields: string[];
  fieldLabels: Record<string, string>;
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
    fieldLabels: result.fieldLabels ?? {},
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
