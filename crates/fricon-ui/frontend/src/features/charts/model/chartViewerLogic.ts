import type { ChartSemantics, DatasetDetail } from "../api/types";
import type {
  ChartDataOptions,
  FilterTableData,
  FilterTableRow,
} from "../api/types";
import type {
  ChartView,
  ComplexViewOption,
  XYDrawStyle,
  XYPlotMode,
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
  plotMode: XYPlotMode;
  drawStyle: XYDrawStyle;
  sweepQuantityName: string | null;
  heatmapQuantityName: string | null;
  complexPlaneQuantityName: string | null;
  xyXName: string | null;
  xyYName: string | null;
  heatmapXName: string | null;
  heatmapYName: string | null;
  traceGroupIndexColumnNames: string[];
  sweepIndexColumnName: string | null;
}

export interface ChartColumnOption {
  name: string;
  label: string | null;
  isComplex: boolean;
  isTrace: boolean;
  hiddenByDefault: boolean;
  isChartAxisCandidate: boolean;
  numeric: boolean;
}

export function columnOptionLabel(
  option: Pick<ChartColumnOption, "label" | "name">,
) {
  return option.label ?? stripSemanticPrefix(option.name);
}

function stripSemanticPrefix(value: string) {
  return value.replace(/^column:/, "").replace(/^logicalIndex:/, "");
}

function semanticValueOptions(
  chartSemantics?: ChartSemantics | null,
): ChartColumnOption[] {
  if (!chartSemantics) {
    return [];
  }

  const values = chartSemantics.valueColumns.map((column) => ({
    name: column.id,
    label: column.label ?? column.name,
    isComplex: column.isComplex,
    isTrace: column.isTrace,
    hiddenByDefault: column.hiddenByDefault,
    isChartAxisCandidate: false,
    numeric: false,
  }));
  const visibleValues = values.filter((column) => !column.hiddenByDefault);
  const hiddenValues = values.filter((column) => column.hiddenByDefault);
  return visibleValues.length > 0
    ? [...visibleValues, ...hiddenValues]
    : values;
}

function semanticAxisOptions(
  chartSemantics?: ChartSemantics | null,
): ChartColumnOption[] {
  if (!chartSemantics) {
    return [];
  }

  const seen = new Set<string>();
  const semanticAxisById = new Map(
    chartSemantics.axes.map((axis) => [axis.id, axis]),
  );
  const orderedAxes = [
    ...chartSemantics.chartAxisCandidates.map(
      (axis) => semanticAxisById.get(axis.id) ?? axis,
    ),
    ...chartSemantics.axes,
  ];
  return orderedAxes
    .filter((axis) => {
      if (seen.has(axis.id)) return false;
      seen.add(axis.id);
      return true;
    })
    .map((axis) => ({
      name: axis.id,
      label: axis.label ?? axis.name,
      isComplex: false,
      isTrace: false,
      hiddenByDefault: false,
      isChartAxisCandidate: axis.kind === "column" && !axis.isInferredAxis,
      numeric: axis.numeric,
    }));
}

function pickSelection(
  options: ChartColumnOption[],
  current: string | null,
  defaultIndex = 0,
): string | null {
  if (options.length === 0) return null;
  const found = options.find((option) => option.name === current);
  if (found) return found.name;
  return options[defaultIndex]?.name ?? options[0]?.name ?? null;
}

function pickOptionalSelection(
  options: ChartColumnOption[],
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
  state: ChartViewerSelectionState,
  chartSemantics?: ChartSemantics | null,
) {
  const indexColumns = semanticAxisOptions(chartSemantics);
  const plottedIndexColumns = indexColumns.filter((column) => column.numeric);
  const roleIndexColumns = chartSemantics
    ? indexColumns.filter((column) => !column.isChartAxisCandidate)
    : indexColumns;
  const sweepRoleIndexColumns = roleIndexColumns.filter(
    (column) => column.numeric,
  );
  const valueColumns = semanticValueOptions(chartSemantics);
  const allColumns = [...valueColumns, ...indexColumns];
  const sweepQuantityOptions = valueColumns;
  const heatmapQuantityOptions = valueColumns;
  const complexPlaneQuantityOptions = valueColumns.filter(
    (column) => column.isComplex,
  );
  const scalarXYColumnOptions = valueColumns.filter(
    (column) => !column.isComplex && !column.isTrace,
  );
  const traceXYColumnOptions = valueColumns.filter(
    (column) => !column.isComplex && column.isTrace,
  );

  const availableViews = (() => {
    const views: ChartView[] = [];
    if (valueColumns.length > 0) views.push("xy");
    if (valueColumns.length > 0 && plottedIndexColumns.length > 0)
      views.push("heatmap");
    return views;
  })();

  const effectiveView =
    availableViews.includes(state.view) && availableViews.length > 0
      ? state.view
      : (availableViews[0] ?? state.view);

  const availablePlotModes = (() => {
    const plotModes: { label: string; value: XYPlotMode }[] = [];
    if (sweepQuantityOptions.length > 0) {
      plotModes.push({
        label: "Quantity vs Sweep",
        value: "quantity_vs_sweep",
      });
    }
    if (scalarXYColumnOptions.length >= 2 || traceXYColumnOptions.length >= 2) {
      plotModes.push({ label: "X-Y Plot", value: "xy" });
    }
    if (complexPlaneQuantityOptions.length > 0) {
      plotModes.push({ label: "Complex Plane", value: "complex_plane" });
    }
    return plotModes;
  })();

  const effectivePlotMode = availablePlotModes.some(
    (option) => option.value === state.plotMode,
  )
    ? state.plotMode
    : (availablePlotModes[0]?.value ?? state.plotMode);

  const effectiveSweepQuantityName = pickSelection(
    sweepQuantityOptions,
    state.sweepQuantityName,
  );
  const sweepQuantity = allColumns.find(
    (column) => column.name === effectiveSweepQuantityName,
  );

  const effectiveHeatmapQuantityName = pickSelection(
    heatmapQuantityOptions,
    state.heatmapQuantityName,
  );
  const heatmapQuantity = allColumns.find(
    (column) => column.name === effectiveHeatmapQuantityName,
  );

  const effectiveComplexPlaneQuantityName = pickSelection(
    complexPlaneQuantityOptions,
    state.complexPlaneQuantityName,
  );
  const complexPlaneQuantity = allColumns.find(
    (column) => column.name === effectiveComplexPlaneQuantityName,
  );

  const xyXOptions =
    scalarXYColumnOptions.length >= 2 && traceXYColumnOptions.length >= 2
      ? [...scalarXYColumnOptions, ...traceXYColumnOptions]
      : scalarXYColumnOptions.length >= 2
        ? scalarXYColumnOptions
        : traceXYColumnOptions;
  const effectiveXYXName = pickSelection(xyXOptions, state.xyXName);
  const xyXColumn = allColumns.find(
    (column) => column.name === effectiveXYXName,
  );
  const xyYOptions = xyXColumn?.isTrace
    ? traceXYColumnOptions
    : scalarXYColumnOptions;
  const effectiveXYYName = pickSelection(
    xyYOptions.filter((column) => column.name !== effectiveXYXName),
    state.xyYName,
  );
  const xyYColumn = allColumns.find(
    (column) => column.name === effectiveXYYName,
  );

  const heatmapXOptions = plottedIndexColumns;
  const heatmapYOptions = plottedIndexColumns;
  const effectiveHeatmapXName = pickSelection(
    heatmapXOptions,
    state.heatmapXName,
    heatmapXOptions.length - 1,
  );
  const heatmapYSelectionOptions = heatmapQuantity?.isTrace
    ? heatmapYOptions
    : heatmapYOptions.filter((column) => column.name !== effectiveHeatmapXName);
  const heatmapYDefaultIndex = Math.max(heatmapYSelectionOptions.length - 1, 0);
  const effectiveHeatmapYName = pickSelection(
    heatmapYSelectionOptions,
    state.heatmapYName,
    heatmapYDefaultIndex,
  );
  const heatmapXColumn = allColumns.find(
    (column) => column.name === effectiveHeatmapXName,
  );
  const heatmapYColumn = allColumns.find(
    (column) => column.name === effectiveHeatmapYName,
  );

  const activeXYSource = (() => {
    if (effectivePlotMode === "quantity_vs_sweep") return sweepQuantity;
    if (effectivePlotMode === "complex_plane") return complexPlaneQuantity;
    return xyXColumn;
  })();

  const xyUsesTraceSource =
    effectiveView === "xy" &&
    ((effectivePlotMode === "quantity_vs_sweep" &&
      Boolean(sweepQuantity?.isTrace)) ||
      (effectivePlotMode === "complex_plane" &&
        Boolean(complexPlaneQuantity?.isTrace)) ||
      (effectivePlotMode === "xy" && Boolean(xyXColumn?.isTrace)));
  const liveMonitorUsesForcedRoles =
    effectiveView === "xy" &&
    !xyUsesTraceSource &&
    activeXYSource !== undefined &&
    roleIndexColumns.length > 0;
  const liveMonitorSweepIndexColumnName = liveMonitorUsesForcedRoles
    ? (sweepRoleIndexColumns[sweepRoleIndexColumns.length - 1]?.name ?? null)
    : null;
  const liveMonitorTraceGroupIndexColumnNames =
    liveMonitorUsesForcedRoles && roleIndexColumns.length > 1
      ? roleIndexColumns
          .filter((column) => column.name !== liveMonitorSweepIndexColumnName)
          .map((column) => column.name)
      : [];

  const xyRoleControlsVisible =
    effectiveView === "xy" &&
    !xyUsesTraceSource &&
    roleIndexColumns.length > 0 &&
    activeXYSource !== undefined;

  const sweepAxisOptions = xyRoleControlsVisible
    ? sweepRoleIndexColumns.filter(
        (column) => !state.traceGroupIndexColumnNames.includes(column.name),
      )
    : [];

  const sweepAxisFallback = "last" as const;
  const effectiveSweepIndexColumnName = xyRoleControlsVisible
    ? pickOptionalSelection(
        sweepAxisOptions,
        state.sweepIndexColumnName,
        sweepAxisFallback,
      )
    : null;
  const sweepAxisColumn = allColumns.find(
    (column) => column.name === effectiveSweepIndexColumnName,
  );

  const traceGroupOptions = xyRoleControlsVisible
    ? roleIndexColumns.filter(
        (column) => column.name !== effectiveSweepIndexColumnName,
      )
    : [];
  const effectiveTraceGroupIndexColumnNames = traceGroupOptions
    .filter((column) => state.traceGroupIndexColumnNames.includes(column.name))
    .map((column) => column.name);

  const effectiveDrawStyle =
    effectiveView === "heatmap" ? null : state.drawStyle;

  const complexControlsDisabled = (() => {
    if (effectiveView === "heatmap") return !heatmapQuantity?.isComplex;
    if (effectiveView === "xy" && effectivePlotMode === "quantity_vs_sweep") {
      return !sweepQuantity?.isComplex;
    }
    return true;
  })();

  const excludeColumns = (() => {
    if (effectiveView === "heatmap") {
      const excludes: string[] = [];
      if (heatmapQuantity?.isTrace) {
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
      ...effectiveTraceGroupIndexColumnNames,
      ...(effectiveSweepIndexColumnName ? [effectiveSweepIndexColumnName] : []),
    ];
  })();

  return {
    availableViews,
    availablePlotModes,
    drawStyleOptions,
    effectiveView,
    effectivePlotMode,
    effectiveDrawStyle,
    sweepQuantityOptions,
    heatmapQuantityOptions,
    complexPlaneQuantityOptions,
    scalarXYColumnOptions,
    traceXYColumnOptions,
    xyXOptions,
    xyYOptions: xyYOptions.filter((column) => column.name !== effectiveXYXName),
    heatmapXOptions,
    heatmapYOptions: heatmapYSelectionOptions,
    effectiveSweepQuantityName,
    effectiveHeatmapQuantityName,
    effectiveComplexPlaneQuantityName,
    effectiveXYXName,
    effectiveXYYName,
    effectiveHeatmapXName,
    effectiveHeatmapYName,
    sweepQuantity,
    heatmapQuantity,
    complexPlaneQuantity,
    xyXColumn,
    xyYColumn,
    heatmapXColumn,
    heatmapYColumn,
    xyUsesTraceSource,
    liveMonitorUsesForcedRoles,
    liveMonitorSweepIndexColumnName,
    liveMonitorTraceGroupIndexColumnNames,
    xyRoleControlsVisible,
    sweepAxisOptions,
    effectiveSweepIndexColumnName,
    sweepAxisColumn,
    traceGroupOptions,
    effectiveTraceGroupIndexColumnNames,
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
    if (!derived.heatmapQuantity || !derived.heatmapYColumn) {
      return null;
    }
    if (!derived.heatmapQuantity.isTrace && !derived.heatmapXColumn) {
      return null;
    }
    return {
      view: "heatmap",
      quantity: derived.heatmapQuantity.name,
      xColumn: derived.heatmapQuantity.isTrace
        ? undefined
        : (derived.heatmapXColumn?.name ?? undefined),
      yColumn: derived.heatmapYColumn.name,
      complexViewSingle: derived.heatmapQuantity.isComplex
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
        traceGroupIndexColumns:
          derived.effectiveTraceGroupIndexColumnNames.length > 0
            ? derived.effectiveTraceGroupIndexColumnNames
            : undefined,
        sweepIndexColumn: derived.effectiveSweepIndexColumnName ?? undefined,
      }
    : {};

  if (
    derived.effectivePlotMode === "quantity_vs_sweep" &&
    derived.sweepQuantity
  ) {
    return {
      view: "xy",
      plotMode: "quantity_vs_sweep",
      drawStyle: derived.effectiveDrawStyle,
      quantity: derived.sweepQuantity.name,
      complexViews: derived.sweepQuantity.isComplex
        ? selectedComplexView
        : undefined,
      indexFilters,
      excludeColumns: derived.excludeColumns,
      ...roleOptions,
    };
  }

  if (
    derived.effectivePlotMode === "xy" &&
    derived.xyXColumn &&
    derived.xyYColumn
  ) {
    return {
      view: "xy",
      plotMode: "xy",
      drawStyle: derived.effectiveDrawStyle,
      xColumn: derived.xyXColumn.name,
      yColumn: derived.xyYColumn.name,
      indexFilters,
      excludeColumns: derived.excludeColumns,
      ...roleOptions,
    };
  }

  if (
    derived.effectivePlotMode === "complex_plane" &&
    derived.complexPlaneQuantity
  ) {
    return {
      view: "xy",
      plotMode: "complex_plane",
      drawStyle: derived.effectiveDrawStyle,
      quantity: derived.complexPlaneQuantity.name,
      indexFilters,
      excludeColumns: derived.excludeColumns,
      ...roleOptions,
    };
  }

  return null;
}
