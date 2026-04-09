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
});
