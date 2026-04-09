import type { ColumnInfo, DatasetDetail } from "../api/types";
import type {
  ChartDataOptions,
  FilterTableData,
  FilterTableRow,
} from "../api/types";
import type {
  ChartView,
  ComplexViewOption,
  XYDrawStyle,
  XYProjection,
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
  view: ChartView;
  projection: XYProjection;
  drawStyle: XYDrawStyle;
  trendSeriesName: string | null;
  heatmapSeriesName: string | null;
  complexXYSeriesName: string | null;
  xyXName: string | null;
  xyYName: string | null;
  heatmapXName: string | null;
  heatmapYName: string | null;
  groupByIndexColumnNames: string[];
  orderByIndexColumnName: string | null;
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

function pickOptionalSelection(
  options: ColumnInfo[],
  current: string | null,
  fallback: "last" | null,
): string | null {
  if (options.length === 0) return null;
  const found = options.find((option) => option.name === current);
  if (found) return found.name;
  if (fallback === "last") {
    return options[options.length - 1]?.name ?? null;
  }
  return null;
}

const drawStyleOptions: { label: string; value: XYDrawStyle }[] = [
  { label: "Line", value: "line" },
  { label: "Points", value: "points" },
  { label: "Line + Points", value: "line_points" },
];

export function deriveChartViewerState(
  columns: ColumnInfo[],
  state: ChartViewerSelectionState,
) {
  const indexColumns = columns.filter((column) => column.isIndex);
  const nonIndexColumns = columns.filter((column) => !column.isIndex);
  const trendSeriesOptions = nonIndexColumns;
  const heatmapSeriesOptions = nonIndexColumns;
  const complexXYSeriesOptions = nonIndexColumns.filter(
    (column) => column.isComplex,
  );
  const scalarXYColumnOptions = nonIndexColumns.filter(
    (column) => !column.isComplex && !column.isTrace,
  );
  const traceXYColumnOptions = nonIndexColumns.filter(
    (column) => !column.isComplex && column.isTrace,
  );

  const availableViews = (() => {
    const views: ChartView[] = [];
    if (nonIndexColumns.length > 0) views.push("xy");
    if (nonIndexColumns.length > 0 && indexColumns.length > 0)
      views.push("heatmap");
    return views;
  })();

  const effectiveView =
    availableViews.includes(state.view) && availableViews.length > 0
      ? state.view
      : (availableViews[0] ?? state.view);

  const availableProjections = (() => {
    const projections: { label: string; value: XYProjection }[] = [];
    if (trendSeriesOptions.length > 0) {
      projections.push({ label: "Trend", value: "trend" });
    }
    if (scalarXYColumnOptions.length >= 2 || traceXYColumnOptions.length >= 2) {
      projections.push({ label: "X-Y", value: "xy" });
    }
    if (complexXYSeriesOptions.length > 0) {
      projections.push({ label: "Complex Plane", value: "complex_xy" });
    }
    return projections;
  })();

  const effectiveProjection = availableProjections.some(
    (option) => option.value === state.projection,
  )
    ? state.projection
    : (availableProjections[0]?.value ?? state.projection);

  const effectiveTrendSeriesName = pickSelection(
    trendSeriesOptions,
    state.trendSeriesName,
  );
  const trendSeries = columns.find(
    (column) => column.name === effectiveTrendSeriesName,
  );

  const effectiveHeatmapSeriesName = pickSelection(
    heatmapSeriesOptions,
    state.heatmapSeriesName,
  );
  const heatmapSeries = columns.find(
    (column) => column.name === effectiveHeatmapSeriesName,
  );

  const effectiveComplexXYSeriesName = pickSelection(
    complexXYSeriesOptions,
    state.complexXYSeriesName,
  );
  const complexXYSeries = columns.find(
    (column) => column.name === effectiveComplexXYSeriesName,
  );

  const defaultXYXOptions =
    scalarXYColumnOptions.length >= 2
      ? scalarXYColumnOptions
      : traceXYColumnOptions;
  const effectiveXYXName = pickSelection(defaultXYXOptions, state.xyXName);
  const xyXColumn = columns.find((column) => column.name === effectiveXYXName);
  const xyYOptions = xyXColumn?.isTrace
    ? traceXYColumnOptions
    : scalarXYColumnOptions;
  const effectiveXYYName = pickSelection(
    xyYOptions.filter((column) => column.name !== effectiveXYXName),
    state.xyYName,
  );
  const xyYColumn = columns.find((column) => column.name === effectiveXYYName);

  const heatmapXOptions = indexColumns;
  const heatmapYOptions = indexColumns;
  const effectiveHeatmapXName = pickSelection(
    heatmapXOptions,
    state.heatmapXName,
    heatmapXOptions.length - 1,
  );
  const heatmapYSelectionOptions = heatmapSeries?.isTrace
    ? heatmapYOptions
    : heatmapYOptions.filter((column) => column.name !== effectiveHeatmapXName);
  const heatmapYDefaultIndex = Math.max(heatmapYSelectionOptions.length - 1, 0);
  const effectiveHeatmapYName = pickSelection(
    heatmapYSelectionOptions,
    state.heatmapYName,
    heatmapYDefaultIndex,
  );
  const heatmapXColumn = columns.find(
    (column) => column.name === effectiveHeatmapXName,
  );
  const heatmapYColumn = columns.find(
    (column) => column.name === effectiveHeatmapYName,
  );

  const activeXYSource = (() => {
    if (effectiveProjection === "trend") return trendSeries;
    if (effectiveProjection === "complex_xy") return complexXYSeries;
    return xyXColumn;
  })();

  const xyUsesTraceSource =
    effectiveView === "xy" &&
    ((effectiveProjection === "trend" && Boolean(trendSeries?.isTrace)) ||
      (effectiveProjection === "complex_xy" &&
        Boolean(complexXYSeries?.isTrace)) ||
      (effectiveProjection === "xy" && Boolean(xyXColumn?.isTrace)));

  const xyRoleControlsVisible =
    effectiveView === "xy" &&
    !xyUsesTraceSource &&
    indexColumns.length > 0 &&
    activeXYSource !== undefined;

  const orderByOptions = xyRoleControlsVisible
    ? indexColumns.filter(
        (column) => !state.groupByIndexColumnNames.includes(column.name),
      )
    : [];

  const orderByFallback = "last";
  const effectiveOrderByIndexColumnName = xyRoleControlsVisible
    ? pickOptionalSelection(
        orderByOptions,
        state.orderByIndexColumnName,
        orderByFallback,
      )
    : null;
  const orderByColumn = columns.find(
    (column) => column.name === effectiveOrderByIndexColumnName,
  );

  const groupByOptions = xyRoleControlsVisible
    ? indexColumns.filter(
        (column) => column.name !== effectiveOrderByIndexColumnName,
      )
    : [];
  const effectiveGroupByIndexColumnNames = groupByOptions
    .filter((column) => state.groupByIndexColumnNames.includes(column.name))
    .map((column) => column.name);

  const effectiveDrawStyle =
    effectiveView === "heatmap" ? null : state.drawStyle;

  const complexControlsDisabled = (() => {
    if (effectiveView === "heatmap") return !heatmapSeries?.isComplex;
    if (effectiveView === "xy" && effectiveProjection === "trend") {
      return !trendSeries?.isComplex;
    }
    return true;
  })();

  const excludeColumns = (() => {
    if (effectiveView === "heatmap") {
      const excludes: string[] = [];
      if (heatmapSeries?.isTrace) {
        if (heatmapYColumn) excludes.push(heatmapYColumn.name);
      } else {
        if (heatmapXColumn) excludes.push(heatmapXColumn.name);
        if (heatmapYColumn) excludes.push(heatmapYColumn.name);
      }
      return excludes;
    }

    if (!xyRoleControlsVisible) {
      return [];
    }

    return [
      ...effectiveGroupByIndexColumnNames,
      ...(effectiveOrderByIndexColumnName
        ? [effectiveOrderByIndexColumnName]
        : []),
    ];
  })();

  return {
    availableViews,
    availableProjections,
    drawStyleOptions,
    effectiveView,
    effectiveProjection,
    effectiveDrawStyle,
    trendSeriesOptions,
    heatmapSeriesOptions,
    complexXYSeriesOptions,
    scalarXYColumnOptions,
    traceXYColumnOptions,
    xyXOptions: defaultXYXOptions,
    xyYOptions: xyYOptions.filter((column) => column.name !== effectiveXYXName),
    heatmapXOptions,
    heatmapYOptions: heatmapYSelectionOptions,
    effectiveTrendSeriesName,
    effectiveHeatmapSeriesName,
    effectiveComplexXYSeriesName,
    effectiveXYXName,
    effectiveXYYName,
    effectiveHeatmapXName,
    effectiveHeatmapYName,
    trendSeries,
    heatmapSeries,
    complexXYSeries,
    xyXColumn,
    xyYColumn,
    heatmapXColumn,
    heatmapYColumn,
    xyUsesTraceSource,
    xyRoleControlsVisible,
    orderByOptions,
    effectiveOrderByIndexColumnName,
    orderByColumn,
    groupByOptions,
    effectiveGroupByIndexColumnNames,
    excludeColumns,
    complexControlsDisabled,
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

  if (derived.effectiveView === "heatmap") {
    if (!derived.heatmapSeries || !derived.heatmapYColumn) {
      return null;
    }
    if (!derived.heatmapSeries.isTrace && !derived.heatmapXColumn) {
      return null;
    }
    return {
      view: "heatmap",
      series: derived.heatmapSeries.name,
      xColumn: derived.heatmapSeries.isTrace
        ? undefined
        : (derived.heatmapXColumn?.name ?? undefined),
      yColumn: derived.heatmapYColumn.name,
      complexViewSingle: derived.heatmapSeries.isComplex
        ? selectedComplexViewSingle
        : undefined,
      indexFilters,
      excludeColumns: derived.excludeColumns,
    };
  }

  if (!derived.effectiveDrawStyle) {
    return null;
  }

  const roleOptions = derived.xyRoleControlsVisible
    ? {
        groupByIndexColumns:
          derived.effectiveGroupByIndexColumnNames.length > 0
            ? derived.effectiveGroupByIndexColumnNames
            : undefined,
        orderByIndexColumn:
          derived.effectiveOrderByIndexColumnName ?? undefined,
      }
    : {};

  if (derived.effectiveProjection === "trend" && derived.trendSeries) {
    return {
      view: "xy",
      projection: "trend",
      drawStyle: derived.effectiveDrawStyle,
      series: derived.trendSeries.name,
      complexViews: derived.trendSeries.isComplex
        ? selectedComplexView
        : undefined,
      indexFilters,
      excludeColumns: derived.excludeColumns,
      ...roleOptions,
    };
  }

  if (
    derived.effectiveProjection === "xy" &&
    derived.xyXColumn &&
    derived.xyYColumn
  ) {
    return {
      view: "xy",
      projection: "xy",
      drawStyle: derived.effectiveDrawStyle,
      xColumn: derived.xyXColumn.name,
      yColumn: derived.xyYColumn.name,
      indexFilters,
      excludeColumns: derived.excludeColumns,
      ...roleOptions,
    };
  }

  if (derived.effectiveProjection === "complex_xy" && derived.complexXYSeries) {
    return {
      view: "xy",
      projection: "complex_xy",
      drawStyle: derived.effectiveDrawStyle,
      series: derived.complexXYSeries.name,
      indexFilters,
      excludeColumns: derived.excludeColumns,
      ...roleOptions,
    };
  }

  return null;
}
