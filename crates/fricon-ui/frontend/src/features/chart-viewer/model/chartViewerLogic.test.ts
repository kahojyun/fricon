import { describe, expect, it } from "vitest";
import type {
  ColumnInfo,
  DatasetDetail,
  FilterTableData,
  FilterTableRow,
} from "@/shared/lib/backend";
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
    chartType: "line",
    seriesName: null,
    xColumnName: null,
    yColumnName: null,
    scatterMode: "complex",
    scatterSeriesName: null,
    scatterTraceXName: null,
    scatterTraceYName: null,
    scatterXName: null,
    scatterYName: null,
    scatterBinName: null,
    ...overrides,
  };
}

function makeDatasetDetail(columns: ColumnInfo[]): DatasetDetail {
  return {
    id: 1,
    name: "Dataset",
    description: "",
    favorite: false,
    tags: [],
    status: "Completed",
    createdAt: new Date("2026-01-01T00:00:00Z"),
    columns,
  } as DatasetDetail;
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
  it("selects trailing index columns as default X/Y", () => {
    const columns = [
      makeColumn({ name: "idxA", isIndex: true }),
      makeColumn({ name: "idxB", isIndex: true }),
      makeColumn({ name: "signal" }),
    ];

    const derived = deriveChartViewerState(columns, makeState());

    expect(derived.effectiveXColumnName).toBe("idxB");
    expect(derived.effectiveYColumnName).toBe("idxA");
  });

  it("falls back scatter mode to available option", () => {
    const columns = [makeColumn({ name: "c", isComplex: true })];

    const derived = deriveChartViewerState(
      columns,
      makeState({ chartType: "scatter", scatterMode: "trace_xy" }),
    );

    expect(derived.effectiveScatterMode).toBe("complex");
    expect(derived.scatterModeOptions.map((item) => item.value)).toEqual([
      "complex",
    ]);
  });

  it("computes scatter exclusion from selected bin column", () => {
    const columns = [
      makeColumn({ name: "idxA", isIndex: true }),
      makeColumn({ name: "idxB", isIndex: true }),
      makeColumn({ name: "xVal" }),
      makeColumn({ name: "yVal" }),
    ];

    const derived = deriveChartViewerState(
      columns,
      makeState({ chartType: "scatter", scatterMode: "xy" }),
    );

    expect(derived.effectiveScatterBinName).toBe("idxB");
    expect(derived.excludeColumns).toEqual(["idxB"]);
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

  it("builds scatter xy request with derived bin exclusion", () => {
    const columns = [
      makeColumn({ name: "idxA", isIndex: true }),
      makeColumn({ name: "idxB", isIndex: true }),
      makeColumn({ name: "xVal" }),
      makeColumn({ name: "yVal" }),
    ];
    const derived = deriveChartViewerState(
      columns,
      makeState({ chartType: "scatter", scatterMode: "xy" }),
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
      chartType: "scatter",
      scatter: {
        mode: "xy",
        xColumn: "xVal",
        yColumn: "yVal",
        binColumn: "idxB",
      },
      indexFilters: [1],
      excludeColumns: ["idxB"],
    });
  });
});
