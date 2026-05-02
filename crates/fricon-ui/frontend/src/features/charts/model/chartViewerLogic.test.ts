import { describe, expect, it } from "vitest";
import type {
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
    isIndex: false,
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
      makeColumn({ name: "idxA", isIndex: true }),
      makeColumn({ name: "idxB", isIndex: true }),
      makeColumn({ name: "signal" }),
    ];

    const derived = deriveChartViewerState(columns, makeState());

    expect(derived.effectiveSweepIndexColumnName).toBe("idxB");
  });

  it("keeps the same default order-by when style changes", () => {
    const columns = [
      makeColumn({ name: "idxA", isIndex: true }),
      makeColumn({ name: "idxB", isIndex: true }),
      makeColumn({ name: "signal" }),
    ];

    const lineDerived = deriveChartViewerState(
      columns,
      makeState({ plotMode: "complex_plane", drawStyle: "line" }),
    );
    const pointsDerived = deriveChartViewerState(
      columns,
      makeState({ plotMode: "complex_plane", drawStyle: "points" }),
    );

    expect(lineDerived.effectiveSweepIndexColumnName).toBe("idxB");
    expect(pointsDerived.effectiveSweepIndexColumnName).toBe("idxB");
  });

  it("defaults scalar heatmap axes to the two trailing index columns", () => {
    const columns = [
      makeColumn({ name: "idxSlow", isIndex: true }),
      makeColumn({ name: "idxMid", isIndex: true }),
      makeColumn({ name: "idxFast", isIndex: true }),
      makeColumn({ name: "signal" }),
    ];

    const derived = deriveChartViewerState(
      columns,
      makeState({ view: "heatmap" }),
    );

    expect(derived.effectiveHeatmapXName).toBe("idxFast");
    expect(derived.effectiveHeatmapYName).toBe("idxMid");
    expect(derived.excludeColumns).toEqual(["idxFast", "idxMid"]);
  });

  it("falls back plot mode to available option", () => {
    const columns = [makeColumn({ name: "c", isComplex: true })];

    const derived = deriveChartViewerState(
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
      makeColumn({ name: "idxA", isIndex: true }),
      makeColumn({ name: "idxB", isIndex: true }),
      makeColumn({ name: "xVal" }),
      makeColumn({ name: "yVal" }),
    ];

    const derived = deriveChartViewerState(
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
      makeColumn({ name: "idxA", isIndex: true }),
      makeColumn({ name: "signal" }),
    ];
    const derived = deriveChartViewerState(columns, makeState());

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
      makeColumn({ name: "idxA", isIndex: true }),
      makeColumn({ name: "idxB", isIndex: true }),
      makeColumn({ name: "xVal" }),
      makeColumn({ name: "yVal" }),
    ];
    const derived = deriveChartViewerState(
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
      makeColumn({ name: "idxA", isIndex: true }),
      makeColumn({ name: "scalarX" }),
      makeColumn({ name: "scalarY" }),
      makeColumn({ name: "traceX", isTrace: true }),
      makeColumn({ name: "traceY", isTrace: true }),
    ];
    const derived = deriveChartViewerState(
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
      makeColumn({ name: "legacyGuess", isIndex: true }),
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
            isCompatibility: false,
            physicalColumn: null,
          },
          {
            id: "logicalIndex:bias",
            name: "bias",
            label: "Bias",
            kind: "logical_index",
            numeric: true,
            isCompatibility: false,
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
            isCompatibility: false,
            physicalColumn: "physicalAxis",
          },
        ],
      },
    );

    expect(derived.effectiveHeatmapQuantityName).toBe("column:signal");
    expect(derived.heatmapQuantityOptions.map((column) => column.name)).toEqual([
      "column:signal",
      "column:hiddenValue",
    ]);
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
          isCompatibility: false,
          physicalColumn: null,
        },
        {
          id: "logicalIndex:bias",
          name: "bias",
          label: "Bias",
          kind: "logical_index",
          numeric: true,
          isCompatibility: false,
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
          isCompatibility: false,
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
            isCompatibility: false,
            physicalColumn: null,
          },
          {
            id: "logicalIndex:bias",
            name: "bias",
            label: "Bias",
            kind: "logical_index",
            numeric: true,
            isCompatibility: false,
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

  it("keeps compatibility index axes available for sweep/group roles", () => {
    const columns = [
      makeColumn({ name: "run", isIndex: true }),
      makeColumn({ name: "step", isIndex: true }),
      makeColumn({ name: "signal" }),
    ];
    const derived = deriveChartViewerState(columns, makeState(), {
      source: "compatibility_inference",
      duplicatePolicy: "compatibility_row_order_placeholder",
      indexRealization: "none",
      axes: [
        {
          id: "column:run",
          name: "run",
          label: null,
          kind: "column",
          numeric: true,
          isCompatibility: true,
          physicalColumn: "run",
        },
        {
          id: "column:step",
          name: "step",
          label: null,
          kind: "column",
          numeric: true,
          isCompatibility: true,
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
          isCompatibility: false,
          physicalColumn: "run",
        },
        {
          id: "column:step",
          name: "step",
          label: null,
          kind: "column",
          numeric: true,
          isCompatibility: false,
          physicalColumn: "step",
        },
      ],
    });

    expect(derived.sweepAxisOptions.map((column) => column.name)).toEqual([
      "column:run",
      "column:step",
    ]);
    expect(derived.effectiveSweepIndexColumnName).toBe("column:step");
    expect(derived.liveMonitorTraceGroupIndexColumnNames).toEqual(["column:run"]);
  });
});
