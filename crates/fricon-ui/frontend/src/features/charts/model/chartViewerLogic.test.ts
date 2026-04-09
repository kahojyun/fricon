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
    projection: "trend",
    drawStyle: "line",
    trendSeriesName: null,
    heatmapSeriesName: null,
    complexXYSeriesName: null,
    xyXName: null,
    xyYName: null,
    heatmapXName: null,
    heatmapYName: null,
    groupByIndexColumnNames: [],
    orderByIndexColumnName: null,
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

    expect(derived.effectiveOrderByIndexColumnName).toBe("idxB");
  });

  it("falls back projection to available option", () => {
    const columns = [makeColumn({ name: "c", isComplex: true })];

    const derived = deriveChartViewerState(
      columns,
      makeState({ projection: "xy" }),
    );

    expect(derived.effectiveProjection).toBe("trend");
    expect(derived.availableProjections.map((item) => item.value)).toEqual([
      "trend",
      "complex_xy",
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
        projection: "xy",
        drawStyle: "line_points",
        xyXName: "xVal",
        xyYName: "yVal",
        groupByIndexColumnNames: ["idxA"],
        orderByIndexColumnName: "idxB",
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
        projection: "xy",
        drawStyle: "line_points",
        xyXName: "xVal",
        xyYName: "yVal",
        groupByIndexColumnNames: ["idxA"],
        orderByIndexColumnName: "idxB",
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
      projection: "xy",
      drawStyle: "line_points",
      xColumn: "xVal",
      yColumn: "yVal",
      groupByIndexColumns: ["idxA"],
      orderByIndexColumn: "idxB",
      indexFilters: [1],
      excludeColumns: ["idxA", "idxB"],
    });
  });
});
