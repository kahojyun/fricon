import type {
  ChartSeries,
  ChartModel,
  HeatmapSeries,
  XYDrawStyle,
  XYPlotMode,
} from "@/shared/lib/chartTypes";
import type { LiveChartAppendOperation, LiveChartUpdate } from "./types";

const MAGIC = "FCHT";
const VERSION = 2;
const HEADER_LENGTH = 16;

const textDecoder = new TextDecoder();

type PayloadKind = 1 | 2 | 3 | 4 | 5;
type SeriesShape = "xy" | "xyz";

interface SeriesMetadata {
  id: string;
  label: string;
  pointCount: number;
  valueCount: number;
}

interface XySnapshotMetadata {
  plotMode: XYPlotMode;
  drawStyle: XYDrawStyle;
  xName: string;
  yName: string | null;
  series: SeriesMetadata[];
}

interface HeatmapSnapshotMetadata {
  xName: string;
  yName: string;
  series: SeriesMetadata[];
}

interface LiveResetMetadata<T> {
  rowCount: number;
  snapshot: T;
}

type LiveAppendOperationMetadata =
  | {
      kind: "append_points";
      seriesId: string;
      shape: SeriesShape;
      pointCount: number;
      valueCount: number;
    }
  | {
      kind: "append_series";
      shape: SeriesShape;
      id: string;
      label: string;
      pointCount: number;
      valueCount: number;
    };

interface LiveAppendMetadata {
  rowCount: number;
  ops: LiveAppendOperationMetadata[];
}

export function decodeChartSnapshot(bytes: Uint8Array): ChartModel {
  const frame = parseFrame(bytes);
  switch (frame.kind) {
    case 1:
      return decodeXySnapshot(frame.metadata, frame.values);
    case 2:
      return decodeHeatmapSnapshot(frame.metadata, frame.values);
    default:
      throw new Error(
        `Expected chart snapshot payload, received kind ${frame.kind}.`,
      );
  }
}

/**
 * Decodes the feature-local `FCHT` chart frame emitted by the Tauri backend.
 *
 * This decoder assumes the transport hands back an aligned `ArrayBuffer`
 * wrapped as a zero-offset `Uint8Array` at the raw invoke boundary. The frame
 * still validates its own numeric offset so payload corruption fails loudly.
 */
export function decodeLiveChartUpdate(bytes: Uint8Array): LiveChartUpdate {
  const frame = parseFrame(bytes);
  switch (frame.kind) {
    case 3: {
      const metadata = readLiveResetMetadata<XySnapshotMetadata>(
        frame.metadata,
        readXySnapshotMetadata,
      );
      return {
        mode: "reset",
        rowCount: metadata.rowCount,
        snapshot: decodeXySnapshot(metadata.snapshot, frame.values),
      };
    }
    case 4: {
      const metadata = readLiveResetMetadata<HeatmapSnapshotMetadata>(
        frame.metadata,
        readHeatmapSnapshotMetadata,
      );
      return {
        mode: "reset",
        rowCount: metadata.rowCount,
        snapshot: decodeHeatmapSnapshot(metadata.snapshot, frame.values),
      };
    }
    case 5:
      return decodeLiveAppend(frame.metadata, frame.values);
    default:
      throw new Error(
        `Expected live chart payload, received kind ${frame.kind}.`,
      );
  }
}

function parseFrame(bytes: Uint8Array): {
  kind: PayloadKind;
  metadata: unknown;
  values: Uint8Array;
} {
  if (bytes.byteLength < HEADER_LENGTH) {
    throw new Error("Chart wire payload is truncated.");
  }

  const magic = textDecoder.decode(bytes.subarray(0, 4));
  if (magic !== MAGIC) {
    throw new Error(`Invalid chart wire magic: ${magic}`);
  }

  const version = bytes[4];
  if (version !== VERSION) {
    throw new Error(`Unsupported chart wire version: ${version}`);
  }

  const kind = bytes[5];
  if (!isPayloadKind(kind)) {
    throw new Error(`Unknown chart wire payload kind: ${kind}`);
  }

  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const metadataLength = view.getUint32(8, true);
  const metadataEnd = HEADER_LENGTH + metadataLength;
  const numericOffset = view.getUint32(12, true);
  if (metadataEnd > bytes.byteLength) {
    throw new Error("Chart wire metadata is truncated.");
  }
  if (numericOffset < metadataEnd || numericOffset > bytes.byteLength) {
    throw new Error("Chart wire numeric payload offset is invalid.");
  }
  if (numericOffset % 8 !== 0) {
    throw new Error(
      "Chart wire numeric payload offset must be 8-byte aligned.",
    );
  }

  const metadataBytes = bytes.subarray(HEADER_LENGTH, metadataEnd);
  let metadata: unknown;
  try {
    metadata = JSON.parse(textDecoder.decode(metadataBytes));
  } catch (error) {
    throw new Error(
      `Failed to parse chart wire metadata: ${(error as Error).message}`,
      { cause: error },
    );
  }

  return {
    kind,
    metadata,
    values: bytes.subarray(numericOffset),
  };
}

function decodeXySnapshot(
  metadataValue: unknown,
  values: Uint8Array,
): Extract<ChartModel, { type: "xy" }> {
  const metadata = readXySnapshotMetadata(metadataValue);
  const reader = createNumericReader(values);
  const snapshot: Extract<ChartModel, { type: "xy" }> = {
    type: "xy",
    plotMode: metadata.plotMode,
    drawStyle: metadata.drawStyle,
    xName: metadata.xName,
    yName: metadata.yName,
    series: metadata.series.map((series) => ({
      ...series,
      values: reader.read(series.valueCount),
    })),
  };
  reader.finish();
  return snapshot;
}

function decodeHeatmapSnapshot(
  metadataValue: unknown,
  values: Uint8Array,
): Extract<ChartModel, { type: "heatmap" }> {
  const metadata = readHeatmapSnapshotMetadata(metadataValue);
  const reader = createNumericReader(values);
  const snapshot: Extract<ChartModel, { type: "heatmap" }> = {
    type: "heatmap",
    xName: metadata.xName,
    yName: metadata.yName,
    series: metadata.series.map((series) => ({
      ...series,
      values: reader.read(series.valueCount),
    })),
  };
  reader.finish();
  return snapshot;
}

function decodeLiveAppend(
  metadataValue: unknown,
  values: Uint8Array,
): LiveChartUpdate {
  const metadata = readLiveAppendMetadata(metadataValue);
  const reader = createNumericReader(values);
  const ops: LiveChartAppendOperation[] = metadata.ops.map((operation) => {
    const decodedValues = reader.read(operation.valueCount);
    if (operation.kind === "append_points") {
      validateShapeValueCount(
        operation.shape,
        operation.pointCount,
        operation.valueCount,
      );
      return {
        kind: "append_points",
        seriesId: operation.seriesId,
        pointCount: operation.pointCount,
        values: decodedValues,
      };
    }

    validateShapeValueCount(
      operation.shape,
      operation.pointCount,
      operation.valueCount,
    );
    return {
      kind: "append_series",
      series:
        operation.shape === "xy"
          ? {
              shape: "xy" as const,
              series: {
                id: operation.id,
                label: operation.label,
                pointCount: operation.pointCount,
                values: decodedValues,
              } satisfies ChartSeries,
            }
          : {
              shape: "xyz" as const,
              series: {
                id: operation.id,
                label: operation.label,
                pointCount: operation.pointCount,
                values: decodedValues,
              } satisfies HeatmapSeries,
            },
    };
  });

  reader.finish();
  return {
    mode: "append",
    rowCount: metadata.rowCount,
    ops,
  };
}

function createNumericReader(bytes: Uint8Array) {
  if (bytes.byteLength % 8 !== 0) {
    throw new Error("Chart wire numeric payload is truncated.");
  }
  if (bytes.byteOffset % 8 !== 0) {
    throw new Error("Chart wire numeric payload is not aligned.");
  }
  const alignedFloatView = new Float64Array(
    bytes.buffer,
    bytes.byteOffset,
    bytes.byteLength / 8,
  );
  let offset = 0;

  return {
    read(valueCount: number): Float64Array {
      const byteLength = valueCount * 8;
      if (offset + byteLength > bytes.byteLength) {
        throw new Error("Chart wire numeric payload is truncated.");
      }

      offset += byteLength;
      return alignedFloatView.subarray((offset - byteLength) / 8, offset / 8);
    },
    finish() {
      if (offset !== bytes.byteLength) {
        throw new Error("Chart wire payload has trailing numeric bytes.");
      }
    },
  };
}

function readXySnapshotMetadata(value: unknown): XySnapshotMetadata {
  const object = readObject(value, "xy snapshot metadata");
  return {
    plotMode: readXYPlotMode(object.plotMode),
    drawStyle: readXYDrawStyle(object.drawStyle),
    xName: readString(object.xName, "xName"),
    yName: readNullableString(object.yName, "yName"),
    series: readSeriesMetadataArray(object.series),
  };
}

function readHeatmapSnapshotMetadata(value: unknown): HeatmapSnapshotMetadata {
  const object = readObject(value, "heatmap snapshot metadata");
  return {
    xName: readString(object.xName, "xName"),
    yName: readString(object.yName, "yName"),
    series: readSeriesMetadataArray(object.series),
  };
}

function readLiveResetMetadata<T>(
  value: unknown,
  readSnapshot: (snapshot: unknown) => T,
): LiveResetMetadata<T> {
  const object = readObject(value, "live reset metadata");
  return {
    rowCount: readNonNegativeInteger(object.rowCount, "rowCount"),
    snapshot: readSnapshot(object),
  };
}

function readLiveAppendMetadata(value: unknown): LiveAppendMetadata {
  const object = readObject(value, "live append metadata");
  return {
    rowCount: readNonNegativeInteger(object.rowCount, "rowCount"),
    ops: readArray(object.ops, "ops").map((item, index) =>
      readLiveAppendOperationMetadata(item, `ops[${index}]`),
    ),
  };
}

function readLiveAppendOperationMetadata(
  value: unknown,
  path: string,
): LiveAppendOperationMetadata {
  const object = readObject(value, path);
  const kind = readLiteral(
    object.kind,
    `${path}.kind`,
    "append_points",
    "append_series",
  );
  if (kind === "append_points") {
    return {
      kind,
      seriesId: readString(object.seriesId, `${path}.seriesId`),
      shape: readSeriesShape(object.shape, `${path}.shape`),
      pointCount: readNonNegativeInteger(
        object.pointCount,
        `${path}.pointCount`,
      ),
      valueCount: readNonNegativeInteger(
        object.valueCount,
        `${path}.valueCount`,
      ),
    };
  }

  return {
    kind,
    shape: readSeriesShape(object.shape, `${path}.shape`),
    id: readString(object.id, `${path}.id`),
    label: readString(object.label, `${path}.label`),
    pointCount: readNonNegativeInteger(object.pointCount, `${path}.pointCount`),
    valueCount: readNonNegativeInteger(object.valueCount, `${path}.valueCount`),
  };
}

function readSeriesMetadataArray(value: unknown): SeriesMetadata[] {
  return readArray(value, "series").map((item, index) =>
    readSeriesMetadata(item, `series[${index}]`),
  );
}

function readSeriesMetadata(value: unknown, path: string): SeriesMetadata {
  const object = readObject(value, path);
  const pointCount = readNonNegativeInteger(
    object.pointCount,
    `${path}.pointCount`,
  );
  const valueCount = readNonNegativeInteger(
    object.valueCount,
    `${path}.valueCount`,
  );
  const shape = inferShapeFromValueCount(pointCount, valueCount, path);
  validateShapeValueCount(shape, pointCount, valueCount);
  return {
    id: readString(object.id, `${path}.id`),
    label: readString(object.label, `${path}.label`),
    pointCount,
    valueCount,
  };
}

function inferShapeFromValueCount(
  pointCount: number,
  valueCount: number,
  path: string,
): SeriesShape {
  if (valueCount === pointCount * 2) {
    return "xy";
  }
  if (valueCount === pointCount * 3) {
    return "xyz";
  }
  throw new Error(`${path}.valueCount does not match XY or XYZ layout.`);
}

function validateShapeValueCount(
  shape: SeriesShape,
  pointCount: number,
  valueCount: number,
) {
  const expected = pointCount * (shape === "xy" ? 2 : 3);
  if (valueCount !== expected) {
    throw new Error(
      `Expected ${expected} values for ${shape} payload, received ${valueCount}.`,
    );
  }
}

function readXYPlotMode(value: unknown): XYPlotMode {
  return readLiteral(
    value,
    "plotMode",
    "quantity_vs_sweep",
    "xy",
    "complex_plane",
  );
}

function readXYDrawStyle(value: unknown): XYDrawStyle {
  return readLiteral(value, "drawStyle", "line", "points", "line_points");
}

function readSeriesShape(value: unknown, path: string): SeriesShape {
  return readLiteral(value, path, "xy", "xyz");
}

function readObject(value: unknown, label: string): Record<string, unknown> {
  if (typeof value === "object" && value !== null && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  throw new Error(`Expected ${label} to be an object.`);
}

function readArray(value: unknown, path: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new Error(`Expected ${path} to be an array.`);
  }
  return value;
}

function readString(value: unknown, path: string): string {
  if (typeof value !== "string") {
    throw new Error(`Expected ${path} to be a string.`);
  }
  return value;
}

function readNullableString(value: unknown, path: string): string | null {
  if (value === null) {
    return null;
  }
  return readString(value, path);
}

function readNonNegativeInteger(value: unknown, path: string): number {
  if (typeof value !== "number" || !Number.isInteger(value) || value < 0) {
    throw new Error(`Expected ${path} to be a non-negative integer.`);
  }
  return value;
}

function readLiteral<T extends string>(
  value: unknown,
  path: string,
  ...allowed: T[]
): T {
  if (typeof value !== "string" || !allowed.includes(value as T)) {
    throw new Error(`Expected ${path} to be one of: ${allowed.join(", ")}.`);
  }
  return value as T;
}

function isPayloadKind(value: number): value is PayloadKind {
  return value >= 1 && value <= 5;
}
