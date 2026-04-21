import { describe, expect, it } from "vitest";
import { decodeChartSnapshot, decodeLiveChartUpdate } from "./wire";
import {
  encodeChartSnapshotForTest,
  encodeLiveChartUpdateForTest,
} from "@/shared/test/chartWire";

describe("chart api wire decoding", () => {
  it("preserves 64-bit precision for snapshot series values", () => {
    const snapshot = encodeChartSnapshotForTest({
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
          values: new Float64Array([1710000000000, 1, 1710000000001, 2]),
        },
      ],
    });

    const result = decodeChartSnapshot(snapshot);

    expect(result.series[0]?.values).toBeInstanceOf(Float64Array);
    expect(Array.from(result.series[0]?.values ?? [])).toEqual([
      1710000000000, 1, 1710000000001, 2,
    ]);
  });

  it("preserves 64-bit precision for live append payloads", () => {
    const response = encodeLiveChartUpdateForTest({
      mode: "append",
      rowCount: 2,
      ops: [
        {
          kind: "append_points",
          seriesId: "signal",
          pointCount: 1,
          values: new Float64Array([1710000000000, 1]),
        },
      ],
    });

    const update = decodeLiveChartUpdate(response);

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

  it("rejects invalid chart wire payload kinds", () => {
    const bytes = encodeChartSnapshotForTest({
      type: "xy",
      plotMode: "xy",
      drawStyle: "points",
      xName: "timestamp",
      yName: "value",
      series: [],
    });
    bytes[5] = 9;

    expect(() => decodeChartSnapshot(bytes)).toThrow(
      "Unknown chart wire payload kind: 9",
    );
  });

  it("rejects unsupported chart wire versions", () => {
    const bytes = encodeChartSnapshotForTest({
      type: "xy",
      plotMode: "xy",
      drawStyle: "points",
      xName: "timestamp",
      yName: "value",
      series: [],
    });
    bytes[4] = 2;

    expect(() => decodeChartSnapshot(bytes)).toThrow(
      "Unsupported chart wire version: 2",
    );
  });

  it("rejects truncated numeric payloads", () => {
    const bytes = encodeChartSnapshotForTest({
      type: "xy",
      plotMode: "xy",
      drawStyle: "points",
      xName: "timestamp",
      yName: "value",
      series: [
        {
          id: "signal",
          label: "signal",
          pointCount: 1,
          values: new Float64Array([1, 2]),
        },
      ],
    }).subarray(0, -1);

    expect(() => decodeChartSnapshot(bytes)).toThrow(
      "Chart wire numeric payload is truncated.",
    );
  });

  it("rejects invalid live payload metadata", () => {
    const bytes = encodeLiveChartUpdateForTest({
      mode: "append",
      rowCount: 1,
      ops: [
        {
          kind: "append_points",
          seriesId: "signal",
          pointCount: 1,
          values: new Float64Array([1, 2]),
        },
      ],
    });
    const metadataLength = new DataView(bytes.buffer).getUint32(6, true);
    const metadata = JSON.parse(
      new TextDecoder().decode(bytes.subarray(10, 10 + metadataLength)),
    ) as {
      ops: Record<string, unknown>[];
    };
    metadata.ops[0] = {
      kind: "append_points",
      pointCount: 1,
      valueCount: 2,
      shape: "xy",
    };
    const metadataBytes = new TextEncoder().encode(JSON.stringify(metadata));
    const patched = new Uint8Array(
      10 + metadataBytes.length + (bytes.length - 10 - metadataLength),
    );
    patched.set(bytes.subarray(0, 10), 0);
    new DataView(patched.buffer).setUint32(6, metadataBytes.length, true);
    patched.set(metadataBytes, 10);
    patched.set(bytes.subarray(10 + metadataLength), 10 + metadataBytes.length);

    expect(() => decodeLiveChartUpdate(patched)).toThrow(
      "Expected ops[0].seriesId to be a string.",
    );
  });
});
