import { describe, expect, it } from "vitest";
import type {
  ChartSemantics,
  ColumnInfo,
  DatasetDetail,
  FilterTableData,
  FilterTableRow,
} from "../api/types";
import {
  buildChartRequest,
  deriveChartViewerState,
  type ChartViewerSelectionState,
} from "./chartViewerLogic";

function makeColumn(
  overrides: Partial<ColumnInfo> & { name: string },
): ColumnInfo {
  const { name, ...rest } = overrides;
  return {
    name,
    isComplex: false,
    isTrace: false,
    isInferredAxis: false,
    ...rest,
  };
}

function makeState(
  overrides: Partial<ChartViewerSelectionState> = {},
): ChartViewerSelectionState {
  return {
    view: "xy",
    plotMode: "quantity_vs_sweep",
    drawStyle: "line",
    sweepQuantityName: null,
    heatmapQuantityName: null,
    complexPlaneQuantityName: null,
    xyXName: null,
    xyYName: null,
    heatmapXName: null,
    heatmapYName: null,
    traceGroupIndexColumnNames: [],
    sweepIndexColumnName: null,
    ...overrides,
  };
}

function makeDatasetDetail(columns: ColumnInfo[]): DatasetDetail {
  return {
    status: "Completed",
    payloadAvailable: true,
    columns,
  };
}

function makeInferredSemantics(columns: ColumnInfo[]): ChartSemantics {
  const axes = columns
    .filter((column) => column.isInferredAxis)
    .map((column) => ({
      id: column.name,
      name: column.name,
      label: column.label ?? null,
      kind: "column" as const,
      numeric: true,
      isInferredAxis: true,
      physicalColumn: column.name,
    }));
  return {
    source: "manifest",
    duplicatePolicy: axes.length > 0 ? "row_order_placeholder" : "latest_by_record_id",
    indexRealization: "none",
    axes,
    valueColumns: columns
      .filter((column) => !column.isInferredAxis)
      .map((column) => ({
        id: column.name,
        name: column.name,
        label: column.label ?? null,
        isComplex: column.isComplex,
        isTrace: column.isTrace,
        hiddenByDefault: column.hiddenByDefault ?? false,
      })),
    chartAxisCandidates: axes,
  };
}

function deriveWithInferredSemantics(
  columns: ColumnInfo[],
  state: ChartViewerSelectionState,
) {
  return deriveChartViewerState(columns, state, makeInferredSemantics(columns));
}

function makeFilterTableData(): FilterTableData {
  return {
    fields: ["idxA"],
    fieldLabels: {},
    rows: [{ index: 1, displayValues: ["1"], valueIndices: [1] }],
    columnUniqueValues: {
      idxA: [{ index: 1, displayValue: "1" }],
    },
  };
}

describe("chartViewerLogic", () => {
  it("defaults trend ordering to the trailing index column", () => {
    const columns = [
      makeColumn({ name: "idxA", isInferredAxis: true }),
      makeColumn({ name: "idxB", isInferredAxis: true }),
      makeColumn({ name: "signal" }),
    ];

    const derived = deriveWithInferredSemantics(columns, makeState());

    expect(derived.effectiveSweepIndexColumnName).toBe("idxB");
  });

  it("keeps the same default order-by when style changes", () => {
    const columns = [
      makeColumn({ name: "idxA", isInferredAxis: true }),
      makeColumn({ name: "idxB", isInferredAxis: true }),
      makeColumn({ name: "signal" }),
    ];

    const lineDerived = deriveWithInferredSemantics(
      columns,
      makeState({ plotMode: "complex_plane", drawStyle: "line" }),
    );
    const pointsDerived = deriveWithInferredSemantics(
      columns,
      makeState({ plotMode: "complex_plane", drawStyle: "points" }),
    );

    expect(lineDerived.effectiveSweepIndexColumnName).toBe("idxB");
    expect(pointsDerived.effectiveSweepIndexColumnName).toBe("idxB");
  });

  it("defaults scalar heatmap axes to the two trailing index columns", () => {
    const columns = [
      makeColumn({ name: "idxSlow", isInferredAxis: true }),
      makeColumn({ name: "idxMid", isInferredAxis: true }),
      makeColumn({ name: "idxFast", isInferredAxis: true }),
      makeColumn({ name: "signal" }),
    ];

    const derived = deriveWithInferredSemantics(
      columns,
      makeState({ view: "heatmap" }),
    );

    expect(derived.effectiveHeatmapXName).toBe("idxFast");
    expect(derived.effectiveHeatmapYName).toBe("idxMid");
    expect(derived.excludeColumns).toEqual(["idxFast", "idxMid"]);
  });

  it("falls back plot mode to available option", () => {
    const columns = [makeColumn({ name: "c", isComplex: true })];

    const derived = deriveWithInferredSemantics(
      columns,
      makeState({ plotMode: "xy" }),
    );

    expect(derived.effectivePlotMode).toBe("quantity_vs_sweep");
    expect(derived.availablePlotModes.map((item) => item.value)).toEqual([
      "quantity_vs_sweep",
      "complex_plane",
    ]);
  });

  it("excludes explicit index roles from filter-table columns", () => {
    const columns = [
      makeColumn({ name: "idxA", isInferredAxis: true }),
      makeColumn({ name: "idxB", isInferredAxis: true }),
      makeColumn({ name: "xVal" }),
      makeColumn({ name: "yVal" }),
    ];

    const derived = deriveWithInferredSemantics(
      columns,
      makeState({
        plotMode: "xy",
        drawStyle: "line_points",
        xyXName: "xVal",
        xyYName: "yVal",
        traceGroupIndexColumnNames: ["idxA"],
        sweepIndexColumnName: "idxB",
      }),
    );

    expect(derived.excludeColumns).toEqual(["idxA", "idxB"]);
  });

  it("returns null request when filters exist but no resolved row", () => {
    const columns = [
      makeColumn({ name: "idxA", isInferredAxis: true }),
      makeColumn({ name: "signal" }),
    ];
    const derived = deriveWithInferredSemantics(columns, makeState());

    const request = buildChartRequest({
      datasetDetail: makeDatasetDetail(columns),
      filterTableData: makeFilterTableData(),
      hasFilters: true,
      filterRow: null,
      selectedComplexView: ["real", "imag"],
      selectedComplexViewSingle: "mag",
      indexFilters: undefined,
      derived,
    });

    expect(request).toBeNull();
  });

  it("builds scalar XY requests with explicit group/order roles", () => {
    const columns = [
      makeColumn({ name: "idxA", isInferredAxis: true }),
      makeColumn({ name: "idxB", isInferredAxis: true }),
      makeColumn({ name: "xVal" }),
      makeColumn({ name: "yVal" }),
    ];
    const derived = deriveWithInferredSemantics(
      columns,
      makeState({
        plotMode: "xy",
        drawStyle: "line_points",
        xyXName: "xVal",
        xyYName: "yVal",
        traceGroupIndexColumnNames: ["idxA"],
        sweepIndexColumnName: "idxB",
      }),
    );
    const filterRow: FilterTableRow = {
      index: 1,
      displayValues: ["1"],
      valueIndices: [1],
    };

    const request = buildChartRequest({
      datasetDetail: makeDatasetDetail(columns),
      filterTableData: makeFilterTableData(),
      hasFilters: true,
      filterRow,
      selectedComplexView: ["real", "imag"],
      selectedComplexViewSingle: "mag",
      indexFilters: filterRow.valueIndices,
      derived,
    });

    expect(request).toEqual({
      view: "xy",
      plotMode: "xy",
      drawStyle: "line_points",
      xColumn: "xVal",
      yColumn: "yVal",
      traceGroupIndexColumns: ["idxA"],
      sweepIndexColumn: "idxB",
      indexFilters: [1],
      excludeColumns: ["idxA", "idxB"],
    });
  });

  it("preserves trace XY selections when scalar XY columns are also available", () => {
    const columns = [
      makeColumn({ name: "idxA", isInferredAxis: true }),
      makeColumn({ name: "scalarX" }),
      makeColumn({ name: "scalarY" }),
      makeColumn({ name: "traceX", isTrace: true }),
      makeColumn({ name: "traceY", isTrace: true }),
    ];
    const derived = deriveWithInferredSemantics(
      columns,
      makeState({
        plotMode: "xy",
        drawStyle: "points",
        xyXName: "traceX",
        xyYName: "traceY",
      }),
    );

    expect(derived.xyXOptions.map((column) => column.name)).toEqual([
      "scalarX",
      "scalarY",
      "traceX",
      "traceY",
    ]);
    expect(derived.effectiveXYXName).toBe("traceX");
    expect(derived.effectiveXYYName).toBe("traceY");

    const request = buildChartRequest({
      datasetDetail: makeDatasetDetail(columns),
      filterTableData: makeFilterTableData(),
      hasFilters: false,
      filterRow: null,
      selectedComplexView: ["real", "imag"],
      selectedComplexViewSingle: "mag",
      indexFilters: undefined,
      derived,
    });

    expect(request).toEqual({
      view: "xy",
      plotMode: "xy",
      drawStyle: "points",
      xColumn: "traceX",
      yColumn: "traceY",
      indexFilters: undefined,
      excludeColumns: [],
    });
  });

  it("uses resolved chart semantics for value and logical axis defaults", () => {
    const columns = [
      makeColumn({ name: "inferredGuess", isInferredAxis: true }),
      makeColumn({ name: "hiddenValue" }),
      makeColumn({ name: "signal" }),
    ];
    const derived = deriveChartViewerState(
      columns,
      makeState({ view: "heatmap" }),
      {
        source: "manifest",
        duplicatePolicy: "latest_by_record_id",
        indexRealization: "implicit",
        axes: [
          {
            id: "logicalIndex:gate",
            name: "gate",
            label: "Gate",
            kind: "logical_index",
            numeric: true,
            isInferredAxis: false,
            physicalColumn: null,
          },
          {
            id: "logicalIndex:bias",
            name: "bias",
            label: "Bias",
            kind: "logical_index",
            numeric: true,
            isInferredAxis: false,
            physicalColumn: null,
          },
        ],
        valueColumns: [
          {
            id: "column:hiddenValue",
            name: "hiddenValue",
            label: "Hidden",
            isComplex: false,
            isTrace: false,
            hiddenByDefault: true,
          },
          {
            id: "column:signal",
            name: "signal",
            label: "Signal",
            isComplex: false,
            isTrace: false,
            hiddenByDefault: false,
          },
        ],
        chartAxisCandidates: [
          {
            id: "column:physicalAxis",
            name: "physicalAxis",
            label: "Physical Axis",
            kind: "column",
            numeric: true,
            isInferredAxis: false,
            physicalColumn: "physicalAxis",
          },
        ],
      },
    );

    expect(derived.effectiveHeatmapQuantityName).toBe("column:signal");
    expect(derived.heatmapQuantityOptions.map((column) => column.name)).toEqual(
      ["column:signal", "column:hiddenValue"],
    );
    expect(derived.heatmapXOptions.map((column) => column.name)).toEqual([
      "column:physicalAxis",
      "logicalIndex:gate",
      "logicalIndex:bias",
    ]);
    expect(derived.effectiveHeatmapXName).toBe("logicalIndex:bias");
    expect(derived.effectiveHeatmapYName).toBe("logicalIndex:gate");
    expect(derived.excludeColumns).toEqual([
      "logicalIndex:bias",
      "logicalIndex:gate",
    ]);
  });

  it("excludes physical chart-axis candidates from sweep/group roles", () => {
    const columns = [
      makeColumn({ name: "physicalAxis" }),
      makeColumn({ name: "signal" }),
    ];
    const derived = deriveChartViewerState(columns, makeState(), {
      source: "manifest",
      duplicatePolicy: "latest_by_record_id",
      indexRealization: "implicit",
      axes: [
        {
          id: "logicalIndex:gate",
          name: "gate",
          label: "Gate",
          kind: "logical_index",
          numeric: true,
          isInferredAxis: false,
          physicalColumn: null,
        },
        {
          id: "logicalIndex:bias",
          name: "bias",
          label: "Bias",
          kind: "logical_index",
          numeric: true,
          isInferredAxis: false,
          physicalColumn: null,
        },
      ],
      valueColumns: [
        {
          id: "column:signal",
          name: "signal",
          label: "Signal",
          isComplex: false,
          isTrace: false,
          hiddenByDefault: false,
        },
      ],
      chartAxisCandidates: [
        {
          id: "column:physicalAxis",
          name: "physicalAxis",
          label: "Physical Axis",
          kind: "column",
          numeric: true,
          isInferredAxis: false,
          physicalColumn: "physicalAxis",
        },
      ],
    });

    expect(derived.liveMonitorTraceGroupIndexColumnNames).toEqual([
      "logicalIndex:gate",
    ]);
    expect(derived.liveMonitorSweepIndexColumnName).toBe("logicalIndex:bias");
    expect(derived.sweepAxisOptions.map((column) => column.name)).toEqual([
      "logicalIndex:gate",
      "logicalIndex:bias",
    ]);
    expect(derived.heatmapXOptions.map((column) => column.name)).toContain(
      "column:physicalAxis",
    );
  });

  it("keeps categorical logical axes available for grouping roles", () => {
    const columns = [
      makeColumn({ name: "gate" }),
      makeColumn({ name: "signal" }),
    ];
    const derived = deriveChartViewerState(
      columns,
      makeState({
        traceGroupIndexColumnNames: ["logicalIndex:gate"],
      }),
      {
        source: "manifest",
        duplicatePolicy: "latest_by_record_id",
        indexRealization: "implicit",
        axes: [
          {
            id: "logicalIndex:gate",
            name: "gate",
            label: "Gate",
            kind: "logical_index",
            numeric: false,
            isInferredAxis: false,
            physicalColumn: null,
          },
          {
            id: "logicalIndex:bias",
            name: "bias",
            label: "Bias",
            kind: "logical_index",
            numeric: true,
            isInferredAxis: false,
            physicalColumn: null,
          },
        ],
        valueColumns: [
          {
            id: "column:signal",
            name: "signal",
            label: "Signal",
            isComplex: false,
            isTrace: false,
            hiddenByDefault: false,
          },
        ],
        chartAxisCandidates: [],
      },
    );

    expect(derived.heatmapXOptions.map((column) => column.name)).toEqual([
      "logicalIndex:bias",
    ]);
    expect(derived.traceGroupOptions.map((column) => column.name)).toContain(
      "logicalIndex:gate",
    );
    expect(derived.effectiveTraceGroupIndexColumnNames).toEqual([
      "logicalIndex:gate",
    ]);
    expect(derived.effectiveSweepIndexColumnName).toBe("logicalIndex:bias");
  });

  it("keeps inferred axis axes available for sweep/group roles", () => {
    const columns = [
      makeColumn({ name: "run", isInferredAxis: true }),
      makeColumn({ name: "step", isInferredAxis: true }),
      makeColumn({ name: "signal" }),
    ];
    const derived = deriveChartViewerState(columns, makeState(), {
      source: "manifest",
      duplicatePolicy: "row_order_placeholder",
      indexRealization: "none",
      axes: [
        {
          id: "column:run",
          name: "run",
          label: null,
          kind: "column",
          numeric: true,
          isInferredAxis: true,
          physicalColumn: "run",
        },
        {
          id: "column:step",
          name: "step",
          label: null,
          kind: "column",
          numeric: true,
          isInferredAxis: true,
          physicalColumn: "step",
        },
      ],
      valueColumns: [
        {
          id: "column:signal",
          name: "signal",
          label: null,
          isComplex: false,
          isTrace: false,
          hiddenByDefault: false,
        },
      ],
      chartAxisCandidates: [
        {
          id: "column:run",
          name: "run",
          label: null,
          kind: "column",
          numeric: true,
          isInferredAxis: true,
          physicalColumn: "run",
        },
        {
          id: "column:step",
          name: "step",
          label: null,
          kind: "column",
          numeric: true,
          isInferredAxis: false,
          physicalColumn: "step",
        },
      ],
    });

    expect(derived.sweepAxisOptions.map((column) => column.name)).toEqual([
      "column:run",
      "column:step",
    ]);
    expect(derived.effectiveSweepIndexColumnName).toBe("column:step");
    expect(derived.liveMonitorTraceGroupIndexColumnNames).toEqual([
      "column:run",
    ]);
  });
});
