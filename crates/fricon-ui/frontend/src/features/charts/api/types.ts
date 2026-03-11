import type {
  ChartDataResponse as WireChartResponse,
  ColumnUniqueValue,
  ColumnInfo,
  DatasetDetail as WireDatasetDetail,
  DatasetChartDataOptions as WireChartDataOptions,
  DatasetWriteStatus,
  FilterTableOptions,
  Row as FilterTableRow,
  TableData as WireFilterTableData,
} from "@/shared/lib/bindings";
import type { ChartOptions, ComplexViewOption } from "@/shared/lib/chartTypes";
import { normalizeCreatedAtDate } from "@/shared/lib/tauri";

export type {
  ColumnInfo,
  ColumnUniqueValue,
  DatasetWriteStatus,
  FilterTableOptions,
  FilterTableRow,
};

export type DatasetDetail = Omit<WireDatasetDetail, "createdAt"> & {
  createdAt: Date;
};

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

export function normalizeChartOptions(result: WireChartResponse): ChartOptions {
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

  if (result.type === "heatmap") {
    if (result.xCategories == null || result.yCategories == null) {
      throw new Error("Missing heatmap categories in backend response");
    }
    return {
      type: "heatmap",
      xName: result.xName,
      yName: result.yName,
      xCategories: result.xCategories,
      yCategories: result.yCategories,
      series: result.series,
    };
  }

  return {
    type: result.type,
    xName: result.xName,
    yName: result.yName,
    series: result.series,
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

export function normalizeDatasetDetail(
  value: WireDatasetDetail,
): DatasetDetail {
  return normalizeCreatedAtDate(value);
}
