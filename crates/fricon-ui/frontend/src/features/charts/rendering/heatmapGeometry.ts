import type { HeatmapSeries } from "@/shared/lib/chartTypes";

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

export const EMPTY_HEATMAP_GEOMETRY: HeatmapGeometry = {
  xMin: 0,
  xMax: 1,
  yMin: 0,
  yMax: 1,
  series: [],
};

export function buildHeatmapGeometry(series: HeatmapSeries[]): HeatmapGeometry {
  const validPoints = series.flatMap((item) =>
    readSeriesPoints(item).filter(
      (point) =>
        Number.isFinite(point.x) &&
        Number.isFinite(point.y) &&
        Number.isFinite(point.z),
    ),
  );

  const xBounds = buildAxisBounds(validPoints.map((point) => point.x));
  const yBounds = buildAxisBounds(validPoints.map((point) => point.y));

  if (xBounds.spanByValue.size === 0 || yBounds.spanByValue.size === 0) {
    return EMPTY_HEATMAP_GEOMETRY;
  }

  return {
    xMin: xBounds.min,
    xMax: xBounds.max,
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
          const xSpan = xBounds.spanByValue.get(point.x);
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
}

export function heatmapDataBounds(series: HeatmapSeries[]) {
  const geometry = buildHeatmapGeometry(series);
  return {
    xMin: geometry.xMin,
    xMax: geometry.xMax,
    yMin: geometry.yMin,
    yMax: geometry.yMax,
  };
}

export function heatmapAxisCenters(series: HeatmapSeries[]) {
  const validPoints = series.flatMap((item) =>
    readSeriesPoints(item).filter(
      (point) =>
        Number.isFinite(point.x) &&
        Number.isFinite(point.y) &&
        Number.isFinite(point.z),
    ),
  );

  return {
    xValues: Array.from(new Set(validPoints.map((point) => point.x))).sort(
      (left, right) => left - right,
    ),
    yValues: Array.from(new Set(validPoints.map((point) => point.y))).sort(
      (left, right) => left - right,
    ),
  };
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
