import type { ChartModel } from "@/shared/lib/chartTypes";
import type { LiveChartUpdate } from "@/features/charts/api/types";

const encoder = new TextEncoder();

type SeriesShape = "xy" | "xyz";

export function encodeChartSnapshotForTest(chart: ChartModel): Uint8Array {
  if (chart.type === "xy") {
    return encodeFrame(
      1,
      {
        plotMode: chart.plotMode,
        drawStyle: chart.drawStyle,
        xName: chart.xName,
        yName: chart.yName,
        series: chart.series.map((series) => ({
          id: series.id,
          label: series.label,
          pointCount: series.pointCount,
          valueCount: series.values.length,
        })),
      },
      chart.series.map((series) => series.values),
    );
  }

  return encodeFrame(
    2,
    {
      xName: chart.xName,
      yName: chart.yName,
      series: chart.series.map((series) => ({
        id: series.id,
        label: series.label,
        pointCount: series.pointCount,
        valueCount: series.values.length,
      })),
    },
    chart.series.map((series) => series.values),
  );
}

export function encodeLiveChartUpdateForTest(
  update: LiveChartUpdate,
): Uint8Array {
  if (update.mode === "reset") {
    if (update.snapshot.type === "xy") {
      return encodeFrame(
        3,
        {
          rowCount: update.rowCount,
          plotMode: update.snapshot.plotMode,
          drawStyle: update.snapshot.drawStyle,
          xName: update.snapshot.xName,
          yName: update.snapshot.yName,
          series: update.snapshot.series.map((series) => ({
            id: series.id,
            label: series.label,
            pointCount: series.pointCount,
            valueCount: series.values.length,
          })),
        },
        update.snapshot.series.map((series) => series.values),
      );
    }

    return encodeFrame(
      4,
      {
        rowCount: update.rowCount,
        xName: update.snapshot.xName,
        yName: update.snapshot.yName,
        series: update.snapshot.series.map((series) => ({
          id: series.id,
          label: series.label,
          pointCount: series.pointCount,
          valueCount: series.values.length,
        })),
      },
      update.snapshot.series.map((series) => series.values),
    );
  }

  return encodeFrame(
    5,
    {
      rowCount: update.rowCount,
      ops: update.ops.map((operation) =>
        operation.kind === "append_points"
          ? {
              kind: "append_points",
              seriesId: operation.seriesId,
              shape: inferShape(operation.pointCount, operation.values.length),
              pointCount: operation.pointCount,
              valueCount: operation.values.length,
            }
          : {
              kind: "append_series",
              shape: operation.series.shape,
              id: operation.series.series.id,
              label: operation.series.series.label,
              pointCount: operation.series.series.pointCount,
              valueCount: operation.series.series.values.length,
            },
      ),
    },
    update.ops.map((operation) =>
      operation.kind === "append_points"
        ? operation.values
        : operation.series.series.values,
    ),
  );
}

function encodeFrame(
  kind: number,
  metadata: unknown,
  values: Float64Array[],
): Uint8Array {
  const metadataBytes = encoder.encode(JSON.stringify(metadata));
  const numericBytes = values.reduce(
    (total, item) => total + item.length * 8,
    0,
  );
  const bytes = new Uint8Array(10 + metadataBytes.length + numericBytes);
  bytes.set(encoder.encode("FCHT"), 0);
  bytes[4] = 1;
  bytes[5] = kind;
  new DataView(bytes.buffer).setUint32(6, metadataBytes.length, true);
  bytes.set(metadataBytes, 10);

  let offset = 10 + metadataBytes.length;
  for (const block of values) {
    const view = new DataView(bytes.buffer, offset, block.length * 8);
    for (let index = 0; index < block.length; index += 1) {
      view.setFloat64(index * 8, block[index] ?? 0, true);
    }
    offset += block.length * 8;
  }

  return bytes;
}

function inferShape(pointCount: number, valueCount: number): SeriesShape {
  if (valueCount === pointCount * 2) {
    return "xy";
  }
  if (valueCount === pointCount * 3) {
    return "xyz";
  }
  throw new Error("Test payload values do not match XY or XYZ layout.");
}
