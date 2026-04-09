import { describe, expect, it } from "vitest";
import { normalizeChartSnapshot, normalizeLiveChartUpdate } from "./types";

describe("chart api types", () => {
  it("preserves 64-bit precision for snapshot series values", () => {
    const result = normalizeChartSnapshot({
      type: "xy",
      projection: "xy",
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
    });

    expect(result.series[0]?.values).toBeInstanceOf(Float64Array);
    expect(Array.from(result.series[0]?.values ?? [])).toEqual([
      1710000000000, 1, 1710000000001, 2,
    ]);
  });

  it("preserves 64-bit precision for live append payloads", () => {
    const update = normalizeLiveChartUpdate({
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
    });

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
});
