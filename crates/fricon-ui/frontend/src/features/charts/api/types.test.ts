import { describe, expect, it } from "vitest";
import { decodeChartSnapshot, decodeLiveChartUpdate } from "./wire";
import {
  encodeChartSnapshotForTest,
  encodeLiveChartUpdateForTest,
} from "@/shared/test/chartWire";

function alignUp(value: number, alignment: number): number {
  const remainder = value % alignment;
  return remainder === 0 ? value : value + alignment - remainder;
}

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
    expect(result.series[0]?.values.buffer).toBe(snapshot.buffer);
    expect(Array.from(result.series[0]?.values ?? [])).toEqual([
      1710000000000, 1, 1710000000001, 2,
    ]);
  });

  it("rejects misaligned numeric payload views", () => {
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
          pointCount: 1,
          values: new Float64Array([1, 2]),
        },
      ],
    });
    const container = new Uint8Array(snapshot.length + 1);
    container.set(snapshot, 1);

    expect(() => decodeChartSnapshot(container.subarray(1))).toThrow(
      "Chart wire numeric payload is not aligned.",
    );
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
    bytes[4] = 3;

    expect(() => decodeChartSnapshot(bytes)).toThrow(
      "Unsupported chart wire version: 3",
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

  it("rejects trailing numeric payload bytes in snapshots", () => {
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
    });
    const extended = new Uint8Array(bytes.byteLength + 8);
    extended.set(bytes, 0);
    new Float64Array(extended.buffer, bytes.byteLength, 1)[0] = 99;

    expect(() => decodeChartSnapshot(extended)).toThrow(
      "Chart wire payload has trailing numeric bytes.",
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
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const metadataLength = view.getUint32(8, true);
    const numericOffset = view.getUint32(12, true);
    const metadata = JSON.parse(
      new TextDecoder().decode(bytes.subarray(16, 16 + metadataLength)),
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
    const patchedNumericOffset = alignUp(16 + metadataBytes.length, 8);
    const patched = new Uint8Array(
      patchedNumericOffset + (bytes.length - numericOffset),
    );
    patched.set(bytes.subarray(0, 16), 0);
    const patchedView = new DataView(patched.buffer);
    patchedView.setUint32(8, metadataBytes.length, true);
    patchedView.setUint32(12, patchedNumericOffset, true);
    patched.set(metadataBytes, 16);
    patched.set(bytes.subarray(numericOffset), patchedNumericOffset);

    expect(() => decodeLiveChartUpdate(patched)).toThrow(
      "Expected ops[0].seriesId to be a string.",
    );
  });
});
