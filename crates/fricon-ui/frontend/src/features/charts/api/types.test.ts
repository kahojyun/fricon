import type {
  ChartSnapshot as WireChartSnapshot,
  LiveChartDataResponse as WireLiveChartResponse,
} from "@/shared/lib/bindings";
import { describe, expect, it } from "vitest";
import { normalizeChartSnapshot, normalizeLiveChartUpdate } from "./types";

describe("chart api types", () => {
  it("preserves 64-bit precision for snapshot series values", () => {
    const snapshot = {
      type: "xy",
      plotMode: "xy",
      drawStyle: "points",
      xName: "timestamp",
      yName: "value",
      series: [
        {
          id: "signal",
          label: "signal",
          pointCount: 2,
          values: [1710000000000, 1, 1710000000001, 2],
        },
      ],
    } satisfies WireChartSnapshot;

    const result = normalizeChartSnapshot(snapshot);

    expect(result.series[0]?.values).toBeInstanceOf(Float64Array);
    expect(Array.from(result.series[0]?.values ?? [])).toEqual([
      1710000000000, 1, 1710000000001, 2,
    ]);
  });

  it("preserves 64-bit precision for live append payloads", () => {
    const response = {
      mode: "append",
      row_count: 2,
      ops: [
        {
          kind: "append_points",
          series_id: "signal",
          point_count: 1,
          values: [1710000000000, 1],
        },
      ],
    } satisfies WireLiveChartResponse;

    const update = normalizeLiveChartUpdate(response);

    expect(update.mode).toBe("append");
    if (update.mode !== "append") {
      throw new Error("expected append update");
    }

    expect(update.ops[0]?.kind).toBe("append_points");
    if (update.ops[0]?.kind === "append_points") {
      expect(update.ops[0].values).toBeInstanceOf(Float64Array);
      expect(Array.from(update.ops[0].values)).toEqual([1710000000000, 1]);
    }
  });

  it("rejects unknown chart snapshot types instead of silently treating them as xy", () => {
    expect(() =>
      normalizeChartSnapshot({
        type: "line",
        xName: "timestamp",
        series: [],
      } as unknown as WireChartSnapshot),
    ).toThrow("Unknown chart snapshot type: line");
  });

  it("rejects unknown live update modes instead of silently treating them as append", () => {
    expect(() =>
      normalizeLiveChartUpdate({
        mode: "delta",
        row_count: 2,
      } as unknown as WireLiveChartResponse),
    ).toThrow("Unknown live chart update mode: delta");
  });
});
