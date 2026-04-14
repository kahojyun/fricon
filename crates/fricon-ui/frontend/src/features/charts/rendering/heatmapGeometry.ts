import type { HeatmapSeries } from "@/shared/lib/chartTypes";

const MIN_COMPARISON_TOLERANCE = 1e-15;
const COMPARISON_TOLERANCE_RATIO = 1e-12;

export interface HeatmapCellGeometry {
  x: number;
  y: number;
  z: number;
  x0: number;
  x1: number;
  y0: number;
  y1: number;
}

export interface HeatmapSeriesGeometry {
  seriesId: string;
  cells: HeatmapCellGeometry[];
}

export interface HeatmapGeometry {
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
  series: HeatmapSeriesGeometry[];
}

export interface HeatmapAxisCenters {
  xValues: number[];
  yValues: number[];
}

export type HeatmapXAxisTopology =
  | "shared_uniform"
  | "row_local_uniform"
  | "jagged";

export interface HeatmapLayout {
  geometry: HeatmapGeometry;
  bounds: {
    xMin: number;
    xMax: number;
    yMin: number;
    yMax: number;
  };
  centers: HeatmapAxisCenters;
  xTopology: HeatmapXAxisTopology;
}

export const EMPTY_HEATMAP_GEOMETRY: HeatmapGeometry = {
  xMin: 0,
  xMax: 1,
  yMin: 0,
  yMax: 1,
  series: [],
};

const EMPTY_HEATMAP_AXIS_CENTERS: HeatmapAxisCenters = {
  xValues: [],
  yValues: [],
};

export function deriveHeatmapLayout(series: HeatmapSeries[]): HeatmapLayout {
  const coordinatePoints = series.flatMap((item) =>
    readSeriesPoints(item).filter(
      (point) => Number.isFinite(point.x) && Number.isFinite(point.y),
    ),
  );

  const xValues = Array.from(
    new Set(coordinatePoints.map((point) => point.x)),
  ).sort((left, right) => left - right);
  const yValues = Array.from(
    new Set(coordinatePoints.map((point) => point.y)),
  ).sort((left, right) => left - right);
  const yBounds = buildAxisBounds(yValues);
  const rowSpans = buildRowSpansByY(yValues, coordinatePoints, xValues);

  if (rowSpans.byY.size === 0 || yBounds.spanByValue.size === 0) {
    return {
      geometry: EMPTY_HEATMAP_GEOMETRY,
      bounds: {
        xMin: EMPTY_HEATMAP_GEOMETRY.xMin,
        xMax: EMPTY_HEATMAP_GEOMETRY.xMax,
        yMin: EMPTY_HEATMAP_GEOMETRY.yMin,
        yMax: EMPTY_HEATMAP_GEOMETRY.yMax,
      },
      centers: EMPTY_HEATMAP_AXIS_CENTERS,
      xTopology: "shared_uniform",
    };
  }

  const geometry = {
    xMin: rowSpans.min,
    xMax: rowSpans.max,
    yMin: yBounds.min,
    yMax: yBounds.max,
    series: series.map((item) => ({
      seriesId: item.id,
      cells: readSeriesPoints(item)
        .filter(
          (point) =>
            Number.isFinite(point.x) &&
            Number.isFinite(point.y) &&
            Number.isFinite(point.z),
        )
        .flatMap((point) => {
          const xSpan = rowSpans.byY.get(point.y)?.spanByValue.get(point.x);
          const ySpan = yBounds.spanByValue.get(point.y);
          if (!xSpan || !ySpan) return [];
          return [
            {
              ...point,
              x0: xSpan[0],
              x1: xSpan[1],
              y0: ySpan[0],
              y1: ySpan[1],
            },
          ];
        }),
    })),
  };

  return {
    geometry,
    bounds: {
      xMin: geometry.xMin,
      xMax: geometry.xMax,
      yMin: geometry.yMin,
      yMax: geometry.yMax,
    },
    centers: {
      xValues:
        rowSpans.xTopology === "shared_uniform"
          ? rowSpans.sharedCenters
          : EMPTY_HEATMAP_AXIS_CENTERS.xValues,
      yValues,
    },
    xTopology: rowSpans.xTopology,
  };
}

export function getHeatmapXTickValues(
  centers: HeatmapAxisCenters,
  xTopology: HeatmapXAxisTopology,
  maxExplicitTicks = 10,
) {
  return xTopology === "shared_uniform" &&
    centers.xValues.length <= maxExplicitTicks
    ? centers.xValues
    : undefined;
}

interface SeriesPoint {
  x: number;
  y: number;
  z: number;
}

interface AxisBounds {
  min: number;
  max: number;
  spanByValue: Map<number, [number, number]>;
}

interface RowSpanState {
  byY: Map<number, AxisBounds>;
  min: number;
  max: number;
  sharedCenters: number[];
  xTopology: HeatmapXAxisTopology;
}

function readSeriesPoints(series: HeatmapSeries): SeriesPoint[] {
  const points: SeriesPoint[] = [];
  for (let i = 0; i < series.pointCount; i++) {
    const offset = i * 3;
    points.push({
      x: series.values[offset] ?? 0,
      y: series.values[offset + 1] ?? 0,
      z: series.values[offset + 2] ?? 0,
    });
  }
  return points;
}

function buildRowSpansByY(
  yValues: number[],
  coordinatePoints: SeriesPoint[],
  globalXValues: number[],
): RowSpanState {
  const xTolerance = deriveXTolerance(globalXValues);
  const xValuesByY = new Map<number, number[]>();
  for (const point of coordinatePoints) {
    const values = xValuesByY.get(point.y);
    if (values) {
      values.push(point.x);
    } else {
      xValuesByY.set(point.y, [point.x]);
    }
  }

  const byY = new Map<number, AxisBounds>();
  const rowCenters: number[][] = [];
  let min = Infinity;
  let max = -Infinity;

  for (const y of yValues) {
    const rowXValues = xValuesByY.get(y);
    if (!rowXValues || rowXValues.length === 0) continue;

    const uniqueRowXValues = Array.from(new Set(rowXValues)).sort(
      (left, right) => left - right,
    );
    const bounds = buildRowAxisBounds(
      uniqueRowXValues,
      globalXValues,
      xTolerance,
    );
    if (bounds.spanByValue.size === 0) continue;

    byY.set(y, bounds);
    rowCenters.push(uniqueRowXValues);
    min = Math.min(min, bounds.min);
    max = Math.max(max, bounds.max);
  }

  const xTopology = classifyXTopology(rowCenters, xTolerance);

  return {
    byY,
    min,
    max,
    sharedCenters:
      rowCenters.length > 0 && xTopology === "shared_uniform"
        ? rowCenters[0]
        : [],
    xTopology,
  };
}

function buildRowAxisBounds(
  values: number[],
  globalValues: number[],
  xTolerance: number,
): AxisBounds {
  if (values.length === 0) {
    return {
      min: 0,
      max: 1,
      spanByValue: new Map(),
    };
  }

  if (values.length === 1) {
    const center = values[0];
    const halfWidth = resolveSingletonHalfWidth(
      center,
      globalValues,
      xTolerance,
    );
    return {
      min: center - halfWidth,
      max: center + halfWidth,
      spanByValue: new Map([
        [center, [center - halfWidth, center + halfWidth]],
      ]),
    };
  }

  return buildAxisBounds(values);
}

function resolveSingletonHalfWidth(
  center: number,
  globalValues: number[],
  xTolerance: number,
) {
  let nearest = Infinity;
  for (const value of globalValues) {
    const distance = Math.abs(value - center);
    if (distance <= xTolerance || distance >= nearest) continue;
    nearest = distance;
  }
  return Number.isFinite(nearest) ? nearest / 2 : 0.5;
}

function classifyXTopology(
  rowCenters: number[][],
  xTolerance: number,
): HeatmapXAxisTopology {
  if (rowCenters.length === 0) return "shared_uniform";

  const firstCenters = rowCenters[0];
  const sharesCommonGrid = rowCenters.every((centers) =>
    arraysAlmostEqual(centers, firstCenters, xTolerance),
  );
  const everyRowUniform = rowCenters.every((centers) =>
    isUniformRow(centers, xTolerance),
  );

  if (sharesCommonGrid && everyRowUniform) {
    return "shared_uniform";
  }
  if (everyRowUniform) {
    return "row_local_uniform";
  }
  return "jagged";
}

function isUniformRow(centers: number[], xTolerance: number) {
  if (centers.length <= 2) return true;

  const baselineStep = centers[1] - centers[0];
  for (let i = 2; i < centers.length; i++) {
    if (!almostEqual(centers[i] - centers[i - 1], baselineStep, xTolerance)) {
      return false;
    }
  }
  return true;
}

function arraysAlmostEqual(
  left: number[],
  right: number[],
  xTolerance: number,
) {
  if (left.length !== right.length) return false;
  for (let i = 0; i < left.length; i++) {
    if (!almostEqual(left[i], right[i], xTolerance)) return false;
  }
  return true;
}

function almostEqual(left: number, right: number, tolerance: number) {
  return Math.abs(left - right) <= tolerance;
}

function deriveXTolerance(values: number[]) {
  if (values.length <= 1) return MIN_COMPARISON_TOLERANCE;

  const span = Math.abs(values[values.length - 1] - values[0]);
  return Math.max(MIN_COMPARISON_TOLERANCE, span * COMPARISON_TOLERANCE_RATIO);
}

function buildAxisBounds(values: number[]): AxisBounds {
  const unique = Array.from(new Set(values)).sort(
    (left, right) => left - right,
  );
  if (unique.length === 0) {
    return {
      min: 0,
      max: 1,
      spanByValue: new Map(),
    };
  }

  if (unique.length === 1) {
    const center = unique[0];
    return {
      min: center - 0.5,
      max: center + 0.5,
      spanByValue: new Map([[center, [center - 0.5, center + 0.5]]]),
    };
  }

  const edges = new Array<number>(unique.length + 1);
  edges[0] = unique[0] - (unique[1] - unique[0]) / 2;
  for (let i = 1; i < unique.length; i++) {
    edges[i] = (unique[i - 1] + unique[i]) / 2;
  }
  edges[unique.length] =
    unique[unique.length - 1] +
    (unique[unique.length - 1] - unique[unique.length - 2]) / 2;

  const spanByValue = new Map<number, [number, number]>();
  for (let i = 0; i < unique.length; i++) {
    spanByValue.set(unique[i], [edges[i], edges[i + 1]]);
  }

  return {
    min: edges[0],
    max: edges[edges.length - 1],
    spanByValue,
  };
}
