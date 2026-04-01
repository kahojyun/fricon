import type { ColumnInfo, DatasetDetail } from "../api/types";
import type {
  ChartDataOptions,
  FilterTableData,
  FilterTableRow,
} from "../api/types";
import type {
  ChartType,
  ComplexViewOption,
  ScatterMode,
} from "@/shared/lib/chartTypes";

export const complexSeriesOptions: ComplexViewOption[] = [
  "real",
  "imag",
  "mag",
  "arg",
];

export function isComplexViewOption(value: string): value is ComplexViewOption {
  return (complexSeriesOptions as readonly string[]).includes(value);
}

export interface ChartViewerSelectionState {
  chartType: ChartType;
  seriesName: string | null;
  xColumnName: string | null;
  yColumnName: string | null;
  scatterMode: ScatterMode;
  scatterSeriesName: string | null;
  scatterTraceXName: string | null;
  scatterTraceYName: string | null;
  scatterXName: string | null;
  scatterYName: string | null;
  scatterBinName: string | null;
}

function pickSelection(
  options: ColumnInfo[],
  current: string | null,
  defaultIndex = 0,
): string | null {
  if (options.length === 0) return null;
  const found = options.find((option) => option.name === current);
  if (found) return found.name;
  return options[defaultIndex]?.name ?? options[0]?.name ?? null;
}

export function deriveChartViewerState(
  columns: ColumnInfo[],
  state: ChartViewerSelectionState,
) {
  const seriesOptions = columns.filter((column) => !column.isIndex);
  const effectiveSeriesName = pickSelection(seriesOptions, state.seriesName);
  const series = columns.find((column) => column.name === effectiveSeriesName);
  const isComplexSeries = Boolean(series?.isComplex);
  const complexControlsDisabled = !isComplexSeries;
  const isTraceSeries = Boolean(series?.isTrace);

  const xColumnOptions = series?.isTrace
    ? []
    : columns.filter((column) => column.isIndex);
  const yColumnOptions = columns.filter((column) => column.isIndex);
  const defaultYColumnIndex =
    xColumnOptions.length > 0
      ? yColumnOptions.length - 2
      : yColumnOptions.length - 1;
  const effectiveXColumnName = pickSelection(
    xColumnOptions,
    state.xColumnName,
    xColumnOptions.length - 1,
  );
  const effectiveYColumnName = pickSelection(
    yColumnOptions,
    state.yColumnName,
    defaultYColumnIndex,
  );
  const xColumn = columns.find(
    (column) => column.name === effectiveXColumnName,
  );
  const yColumn = columns.find(
    (column) => column.name === effectiveYColumnName,
  );

  const scatterComplexOptions = columns.filter(
    (column) => !column.isIndex && column.isComplex,
  );
  const scatterTraceXYOptions = columns.filter(
    (column) => !column.isIndex && !column.isComplex && column.isTrace,
  );
  const scatterXYOptions = columns.filter(
    (column) => !column.isIndex && !column.isComplex && !column.isTrace,
  );

  const hasIndexColumn = columns.some((column) => column.isIndex);
  const canUseScatterComplex = scatterComplexOptions.length > 0;
  const canUseScatterTraceXY = scatterTraceXYOptions.length >= 2;
  const canUseScatterXY = scatterXYOptions.length >= 2 && hasIndexColumn;

  const effectiveScatterMode: ScatterMode = (() => {
    if (state.scatterMode === "complex" && canUseScatterComplex)
      return "complex";
    if (state.scatterMode === "trace_xy" && canUseScatterTraceXY) {
      return "trace_xy";
    }
    if (state.scatterMode === "xy" && canUseScatterXY) return "xy";
    if (canUseScatterComplex) return "complex";
    if (canUseScatterTraceXY) return "trace_xy";
    return "xy";
  })();

  const effectiveScatterSeriesName = pickSelection(
    scatterComplexOptions,
    state.scatterSeriesName,
  );
  const effectiveScatterTraceXName = pickSelection(
    scatterTraceXYOptions,
    state.scatterTraceXName,
  );
  const effectiveScatterTraceYName = pickSelection(
    scatterTraceXYOptions,
    state.scatterTraceYName,
    1,
  );
  const effectiveScatterXName = pickSelection(
    scatterXYOptions,
    state.scatterXName,
  );
  const effectiveScatterYName = pickSelection(
    scatterXYOptions,
    state.scatterYName,
    1,
  );

  const scatterSeries = columns.find(
    (column) => column.name === effectiveScatterSeriesName,
  );
  const scatterTraceXColumn = columns.find(
    (column) => column.name === effectiveScatterTraceXName,
  );
  const scatterTraceYColumn = columns.find(
    (column) => column.name === effectiveScatterTraceYName,
  );
  const scatterXColumn = columns.find(
    (column) => column.name === effectiveScatterXName,
  );
  const scatterYColumn = columns.find(
    (column) => column.name === effectiveScatterYName,
  );

  const scatterIsTraceBased = (() => {
    if (effectiveScatterMode === "trace_xy") return true;
    return effectiveScatterMode === "complex" && scatterSeries?.isTrace;
  })();

  const scatterBinColumnOptions = (() => {
    const excludedNames = new Set(
      [
        scatterSeries?.name,
        scatterXColumn?.name,
        scatterYColumn?.name,
        scatterTraceXColumn?.name,
        scatterTraceYColumn?.name,
      ].filter((name): name is string => Boolean(name)),
    );
    return columns.filter(
      (column) => column.isIndex && !excludedNames.has(column.name),
    );
  })();

  const effectiveScatterBinName = (() => {
    if (scatterIsTraceBased) return null;
    if (effectiveScatterMode !== "xy" && effectiveScatterMode !== "complex") {
      return null;
    }
    return pickSelection(
      scatterBinColumnOptions,
      state.scatterBinName,
      scatterBinColumnOptions.length - 1,
    );
  })();
  const scatterBinColumn = columns.find(
    (column) => column.name === effectiveScatterBinName,
  );

  const scatterModeOptions = (() => {
    const options: { label: string; value: ScatterMode }[] = [];
    if (canUseScatterComplex) {
      options.push({ label: "Complex (real/imag)", value: "complex" });
    }
    if (canUseScatterTraceXY) {
      options.push({ label: "Trace X/Y", value: "trace_xy" });
    }
    if (canUseScatterXY) {
      options.push({ label: "X/Y columns", value: "xy" });
    }
    return options;
  })();

  const availableChartTypes = (() => {
    if (columns.length === 0) return [];
    const hasSeries = columns.some((column) => !column.isIndex);
    const hasIndex = columns.some((column) => column.isIndex);
    const hasComplex = columns.some(
      (column) => !column.isIndex && column.isComplex,
    );
    const realColumns = columns.filter(
      (column) => !column.isIndex && !column.isComplex && !column.isTrace,
    );
    const realTraceColumns = columns.filter(
      (column) => !column.isIndex && !column.isComplex && column.isTrace,
    );
    const canScatter =
      hasComplex || realColumns.length >= 2 || realTraceColumns.length >= 2;
    const types: ChartType[] = [];
    if (hasSeries) types.push("line");
    if (hasSeries && hasIndex) types.push("heatmap");
    if (canScatter) types.push("scatter");
    return types;
  })();

  const effectiveChartType = (() => {
    if (availableChartTypes.length === 0) return state.chartType;
    return availableChartTypes.includes(state.chartType)
      ? state.chartType
      : (availableChartTypes[0] ?? state.chartType);
  })();

  const excludeColumns = (() => {
    const excludes: string[] = [];
    if (effectiveChartType === "line") {
      if (xColumn) excludes.push(xColumn.name);
    } else if (effectiveChartType === "heatmap") {
      if (series?.isTrace) {
        if (yColumn) excludes.push(yColumn.name);
      } else {
        if (xColumn) excludes.push(xColumn.name);
        if (yColumn) excludes.push(yColumn.name);
      }
    } else if (effectiveChartType === "scatter") {
      if (
        (effectiveScatterMode === "xy" || effectiveScatterMode === "complex") &&
        !scatterIsTraceBased &&
        scatterBinColumn?.isIndex
      ) {
        excludes.push(scatterBinColumn.name);
      }
    }
    return excludes;
  })();

  return {
    seriesOptions,
    effectiveSeriesName,
    series,
    isComplexSeries,
    complexControlsDisabled,
    isTraceSeries,
    xColumnOptions,
    yColumnOptions,
    effectiveXColumnName,
    effectiveYColumnName,
    xColumn,
    yColumn,
    scatterComplexOptions,
    scatterTraceXYOptions,
    scatterXYOptions,
    effectiveScatterMode,
    effectiveScatterSeriesName,
    effectiveScatterTraceXName,
    effectiveScatterTraceYName,
    effectiveScatterXName,
    effectiveScatterYName,
    scatterSeries,
    scatterTraceXColumn,
    scatterTraceYColumn,
    scatterXColumn,
    scatterYColumn,
    scatterIsTraceBased,
    scatterBinColumnOptions,
    effectiveScatterBinName,
    scatterBinColumn,
    scatterModeOptions,
    availableChartTypes,
    effectiveChartType,
    excludeColumns,
  };
}

interface BuildChartRequestOptions {
  datasetDetail: DatasetDetail | null;
  filterTableData: FilterTableData | null;
  hasFilters: boolean;
  filterRow: FilterTableRow | null;
  selectedComplexView: ComplexViewOption[];
  selectedComplexViewSingle: ComplexViewOption;
  indexFilters: number[] | undefined;
  derived: ReturnType<typeof deriveChartViewerState>;
}

export function buildChartRequest(
  options: BuildChartRequestOptions,
): ChartDataOptions | null {
  const {
    datasetDetail,
    filterTableData,
    hasFilters,
    filterRow,
    selectedComplexView,
    selectedComplexViewSingle,
    indexFilters,
    derived,
  } = options;
  if (!datasetDetail || !filterTableData) return null;
  if (hasFilters && !filterRow) return null;

  if (derived.effectiveChartType === "scatter") {
    if (derived.effectiveScatterMode === "complex" && !derived.scatterSeries) {
      return null;
    }
    if (
      derived.effectiveScatterMode === "trace_xy" &&
      (!derived.scatterTraceXColumn || !derived.scatterTraceYColumn)
    ) {
      return null;
    }
    if (
      derived.effectiveScatterMode === "xy" &&
      (!derived.scatterXColumn || !derived.scatterYColumn)
    ) {
      return null;
    }
  } else {
    if (!derived.series) return null;
    if (derived.effectiveChartType === "line") {
      if (!derived.series.isTrace && !derived.xColumn) return null;
    } else if (derived.effectiveChartType === "heatmap") {
      if (!derived.yColumn) return null;
      if (!derived.series.isTrace && !derived.xColumn) return null;
    }
  }

  if (derived.effectiveChartType === "line" && derived.series) {
    return {
      chartType: "line",
      series: derived.series.name,
      xColumn: derived.xColumn?.name,
      complexViews: selectedComplexView,
      indexFilters,
      excludeColumns: derived.excludeColumns,
    };
  }

  if (
    derived.effectiveChartType === "heatmap" &&
    derived.series &&
    derived.yColumn
  ) {
    return {
      chartType: "heatmap",
      series: derived.series.name,
      xColumn: derived.xColumn?.name,
      yColumn: derived.yColumn.name,
      complexViewSingle: selectedComplexViewSingle,
      indexFilters,
      excludeColumns: derived.excludeColumns,
    };
  }

  if (derived.effectiveScatterMode === "complex" && derived.scatterSeries) {
    return {
      chartType: "scatter",
      scatter: {
        mode: "complex",
        series: derived.scatterSeries.name,
      },
      indexFilters,
      excludeColumns: derived.excludeColumns,
    };
  }

  if (
    derived.effectiveScatterMode === "trace_xy" &&
    derived.scatterTraceXColumn &&
    derived.scatterTraceYColumn
  ) {
    return {
      chartType: "scatter",
      scatter: {
        mode: "trace_xy",
        traceXColumn: derived.scatterTraceXColumn.name,
        traceYColumn: derived.scatterTraceYColumn.name,
      },
      indexFilters,
      excludeColumns: derived.excludeColumns,
    };
  }

  if (derived.scatterXColumn && derived.scatterYColumn) {
    return {
      chartType: "scatter",
      scatter: {
        mode: "xy",
        xColumn: derived.scatterXColumn.name,
        yColumn: derived.scatterYColumn.name,
        binColumn: derived.scatterBinColumn?.name ?? null,
      },
      indexFilters,
      excludeColumns: derived.excludeColumns,
    };
  }

  return null;
}
