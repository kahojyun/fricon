import { describe, expect, it } from "vitest";
import type { FilterTableData, FilterTableRow } from "../api/types";
import {
  columnId,
  makeColumn,
  makeDatasetDetail,
  makeFilterTableData as makeChartFilterTableData,
  makeInferredSemantics,
  makeSemanticCapabilities,
  makeSemanticDescriptor,
} from "../test-utils";
import type {
  ChartSemanticAxis,
  ChartSemanticColumn,
  ChartSemanticValueKind,
} from "../api/types";
import {
  buildChartRequest,
  deriveChartViewerState,
  type ChartViewerSelectionState,
} from "./chartViewerLogic";

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

function deriveWithInferredSemantics(
  columns: Parameters<typeof makeInferredSemantics>[0],
  state: ChartViewerSelectionState,
) {
  return deriveChartViewerState(state, makeInferredSemantics(columns));
}

function makeFilterTableData(): FilterTableData {
  return makeChartFilterTableData({
    fields: [columnId("idxA")],
    rows: [{ index: 1, displayValues: ["1"], valueIndices: [1] }],
    columnUniqueValues: {
      [columnId("idxA")]: [{ index: 1, displayValue: "1" }],
    },
  });
}

function testAxis(
  input: Omit<ChartSemanticAxis, "semantic" | "capabilities"> & {
    valueKind?: ChartSemanticValueKind;
    shapeKind?: "scalar" | "trace";
  },
): ChartSemanticAxis {
  const { valueKind = "numeric", shapeKind = "scalar", ...axis } = input;
  const semantic = makeSemanticDescriptor({
    valueKind,
    shapeKind,
    role: "logical_index",
  });
  return {
    ...axis,
    semantic,
    capabilities: makeSemanticCapabilities(semantic),
  };
}

function testValueColumn(
  input: Omit<ChartSemanticColumn, "semantic" | "capabilities"> & {
    valueKind?: ChartSemanticValueKind;
    shapeKind?: "scalar" | "trace";
  },
): ChartSemanticColumn {
  const { valueKind = "numeric", shapeKind = "scalar", ...column } = input;
  const semantic = makeSemanticDescriptor({ valueKind, shapeKind });
  return {
    ...column,
    semantic,
    capabilities: makeSemanticCapabilities(semantic),
  };
}

describe("chartViewerLogic", () => {
  it("defaults trend ordering to the trailing inferred axis", () => {
    const columns = [
      makeColumn({ name: "idxA", isInferredAxis: true }),
      makeColumn({ name: "idxB", isInferredAxis: true }),
      makeColumn({ name: "signal" }),
    ];

    const derived = deriveWithInferredSemantics(columns, makeState());

    expect(derived.effectiveSweepIndexColumnName).toBe(columnId("idxB"));
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

    expect(lineDerived.effectiveSweepIndexColumnName).toBe(columnId("idxB"));
    expect(pointsDerived.effectiveSweepIndexColumnName).toBe(columnId("idxB"));
  });

  it("defaults scalar heatmap axes to the two trailing inferred axes", () => {
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

    expect(derived.effectiveHeatmapXName).toBe(columnId("idxFast"));
    expect(derived.effectiveHeatmapYName).toBe(columnId("idxMid"));
    expect(derived.excludeColumns).toEqual([
      columnId("idxFast"),
      columnId("idxMid"),
    ]);
  });

  it("falls back plot mode to available option", () => {
    const columns = [
      makeColumn({
        name: "c",
        semantic: makeSemanticDescriptor({ valueKind: "complex" }),
      }),
    ];

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

  it("excludes explicit semantic axis roles from filter-table columns", () => {
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
        xyXName: columnId("xVal"),
        xyYName: columnId("yVal"),
        traceGroupIndexColumnNames: [columnId("idxA")],
        sweepIndexColumnName: columnId("idxB"),
      }),
    );

    expect(derived.excludeColumns).toEqual([
      columnId("idxA"),
      columnId("idxB"),
    ]);
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
        xyXName: columnId("xVal"),
        xyYName: columnId("yVal"),
        traceGroupIndexColumnNames: [columnId("idxA")],
        sweepIndexColumnName: columnId("idxB"),
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
      xColumn: columnId("xVal"),
      yColumn: columnId("yVal"),
      traceGroupIndexColumns: [columnId("idxA")],
      sweepIndexColumn: columnId("idxB"),
      indexFilters: [1],
      excludeColumns: [columnId("idxA"), columnId("idxB")],
    });
  });

  it("preserves trace XY selections when scalar XY columns are also available", () => {
    const columns = [
      makeColumn({ name: "idxA", isInferredAxis: true }),
      makeColumn({ name: "scalarX" }),
      makeColumn({ name: "scalarY" }),
      makeColumn({
        name: "traceX",
        semantic: makeSemanticDescriptor({ shapeKind: "trace" }),
      }),
      makeColumn({
        name: "traceY",
        semantic: makeSemanticDescriptor({ shapeKind: "trace" }),
      }),
    ];
    const derived = deriveWithInferredSemantics(
      columns,
      makeState({
        plotMode: "xy",
        drawStyle: "points",
        xyXName: columnId("traceX"),
        xyYName: columnId("traceY"),
      }),
    );

    expect(derived.xyXOptions.map((column) => column.name)).toEqual([
      columnId("scalarX"),
      columnId("scalarY"),
      columnId("traceX"),
      columnId("traceY"),
    ]);
    expect(derived.effectiveXYXName).toBe(columnId("traceX"));
    expect(derived.effectiveXYYName).toBe(columnId("traceY"));

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
      xColumn: columnId("traceX"),
      yColumn: columnId("traceY"),
      indexFilters: undefined,
      excludeColumns: [],
    });
  });

  it("uses resolved chart semantics for value and logical axis defaults", () => {
    const derived = deriveChartViewerState(makeState({ view: "heatmap" }), {
      duplicatePolicy: "latest_by_record_id",
      indexRealization: "implicit",
      axes: [
        testAxis({
          id: "logicalIndex:gate",
          name: "gate",
          label: "Gate",
          kind: "logical_index",
          isInferredAxis: false,
          physicalColumn: null,
        }),
        testAxis({
          id: "logicalIndex:bias",
          name: "bias",
          label: "Bias",
          kind: "logical_index",
          isInferredAxis: false,
          physicalColumn: null,
        }),
      ],
      valueColumns: [
        testValueColumn({
          id: "column:hiddenValue",
          name: "hiddenValue",
          label: "Hidden",
          hiddenByDefault: true,
        }),
        testValueColumn({
          id: "column:signal",
          name: "signal",
          label: "Signal",
          hiddenByDefault: false,
        }),
      ],
      chartAxisCandidates: [
        testAxis({
          id: "column:physicalAxis",
          name: "physicalAxis",
          label: "Physical Axis",
          kind: "column",
          isInferredAxis: false,
          physicalColumn: "physicalAxis",
        }),
      ],
    });

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
    const derived = deriveChartViewerState(makeState(), {
      duplicatePolicy: "latest_by_record_id",
      indexRealization: "implicit",
      axes: [
        testAxis({
          id: "logicalIndex:gate",
          name: "gate",
          label: "Gate",
          kind: "logical_index",
          isInferredAxis: false,
          physicalColumn: null,
        }),
        testAxis({
          id: "logicalIndex:bias",
          name: "bias",
          label: "Bias",
          kind: "logical_index",
          isInferredAxis: false,
          physicalColumn: null,
        }),
      ],
      valueColumns: [
        testValueColumn({
          id: "column:signal",
          name: "signal",
          label: "Signal",
          hiddenByDefault: false,
        }),
      ],
      chartAxisCandidates: [
        testAxis({
          id: "column:physicalAxis",
          name: "physicalAxis",
          label: "Physical Axis",
          kind: "column",
          isInferredAxis: false,
          physicalColumn: "physicalAxis",
        }),
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
    const derived = deriveChartViewerState(
      makeState({
        traceGroupIndexColumnNames: ["logicalIndex:gate"],
      }),
      {
        duplicatePolicy: "latest_by_record_id",
        indexRealization: "implicit",
        axes: [
          testAxis({
            id: "logicalIndex:gate",
            name: "gate",
            label: "Gate",
            kind: "logical_index",
            valueKind: "categorical",
            isInferredAxis: false,
            physicalColumn: null,
          }),
          testAxis({
            id: "logicalIndex:bias",
            name: "bias",
            label: "Bias",
            kind: "logical_index",
            isInferredAxis: false,
            physicalColumn: null,
          }),
        ],
        valueColumns: [
          testValueColumn({
            id: "column:signal",
            name: "signal",
            label: "Signal",
            hiddenByDefault: false,
          }),
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

  it("uses semantic kind instead of compatibility booleans for eligibility", () => {
    const derived = deriveChartViewerState(
      makeState({
        plotMode: "xy",
        xyXName: "column:traceX",
        xyYName: "column:traceY",
      }),
      {
        duplicatePolicy: "latest_by_record_id",
        indexRealization: "implicit",
        axes: [
          testAxis({
            id: "logicalIndex:category",
            name: "category",
            label: "Category",
            kind: "logical_index",
            valueKind: "categorical",
            isInferredAxis: false,
            physicalColumn: null,
          }),
          testAxis({
            id: "logicalIndex:step",
            name: "step",
            label: "Step",
            kind: "logical_index",
            isInferredAxis: false,
            physicalColumn: null,
          }),
        ],
        valueColumns: [
          testValueColumn({
            id: "column:traceX",
            name: "traceX",
            label: "Trace X",
            shapeKind: "trace",
            hiddenByDefault: false,
          }),
          testValueColumn({
            id: "column:traceY",
            name: "traceY",
            label: "Trace Y",
            shapeKind: "trace",
            hiddenByDefault: false,
          }),
        ],
        chartAxisCandidates: [],
      },
    );

    expect(derived.heatmapXOptions.map((column) => column.name)).toEqual([
      "logicalIndex:step",
    ]);
    expect(derived.xyXOptions.map((column) => column.name)).toEqual([
      "column:traceX",
      "column:traceY",
    ]);
    expect(derived.xyUsesTraceSource).toBe(true);
  });

  it("keeps complex-valued traces eligible for complex projections", () => {
    const columns = [
      makeColumn({
        name: "complexTrace",
        semantic: makeSemanticDescriptor({
          valueKind: "complex",
          shapeKind: "trace",
        }),
      }),
      makeColumn({
        name: "numericTrace",
        semantic: makeSemanticDescriptor({ shapeKind: "trace" }),
      }),
    ];
    const derived = deriveWithInferredSemantics(
      columns,
      makeState({ sweepQuantityName: columnId("complexTrace") }),
    );

    expect(derived.availablePlotModes.map((item) => item.value)).toContain(
      "complex_plane",
    );
    expect(
      derived.complexPlaneQuantityOptions.map((column) => column.name),
    ).toEqual([columnId("complexTrace")]);
    expect(derived.traceXYColumnOptions.map((column) => column.name)).toEqual([
      columnId("numericTrace"),
    ]);
    expect(derived.complexControlsDisabled).toBe(false);

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

    expect(request).toMatchObject({
      view: "xy",
      plotMode: "quantity_vs_sweep",
      quantity: columnId("complexTrace"),
      complexViews: ["real", "imag"],
    });
  });

  it("keeps inferred axis axes available for sweep/group roles", () => {
    const derived = deriveChartViewerState(makeState(), {
      duplicatePolicy: "row_order_placeholder",
      indexRealization: "none",
      axes: [
        testAxis({
          id: "column:run",
          name: "run",
          label: null,
          kind: "column",
          isInferredAxis: true,
          physicalColumn: "run",
        }),
        testAxis({
          id: "column:step",
          name: "step",
          label: null,
          kind: "column",
          isInferredAxis: true,
          physicalColumn: "step",
        }),
      ],
      valueColumns: [
        testValueColumn({
          id: "column:signal",
          name: "signal",
          label: null,
          hiddenByDefault: false,
        }),
      ],
      chartAxisCandidates: [
        testAxis({
          id: "column:run",
          name: "run",
          label: null,
          kind: "column",
          isInferredAxis: true,
          physicalColumn: "run",
        }),
        testAxis({
          id: "column:step",
          name: "step",
          label: null,
          kind: "column",
          isInferredAxis: false,
          physicalColumn: "step",
        }),
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
