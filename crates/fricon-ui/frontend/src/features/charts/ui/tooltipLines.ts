import {
  getXYPoint,
  getXYZPoint,
  type ChartOptions,
} from "@/shared/lib/chartTypes";
import type { ChartInteractionState } from "../hooks/useWebGLChart";
import {
  invertZoomedLinearRange,
  projectLinearRange,
} from "../rendering/mathUtils";

export function getTooltipLines(
  data: ChartOptions,
  interactionState: ChartInteractionState,
  chartX: number,
  chartY: number,
  chartWidth: number,
  chartHeight: number,
): string[] {
  if (data.type === "line" && interactionState.type === "line") {
    const dataX = invertZoomedLinearRange(
      chartX,
      interactionState.zoomState.translateX,
      interactionState.zoomState.scaleX,
      interactionState.xMin,
      interactionState.xMax,
      0,
      chartWidth,
    );

    return data.series.flatMap((series) => {
      const nearest = findNearestX(series, dataX);
      return nearest
        ? [`${series.label}: (${fmt(nearest.x)}, ${fmt(nearest.y)})`]
        : [];
    });
  }

  if (data.type === "scatter" && interactionState.type === "scatter") {
    let bestDist = Infinity;
    let bestSeries = "";
    let bestPoint: { x: number; y: number } | null = null;

    for (const series of data.series) {
      for (let i = 0; i < series.pointCount; i++) {
        const pt = getXYPoint(series, i);
        const px =
          interactionState.zoomState.translateX +
          projectLinearRange(
            pt.x,
            interactionState.xMin,
            interactionState.xMax,
            0,
            chartWidth,
          ) *
            interactionState.zoomState.scaleX;
        const py =
          interactionState.zoomState.translateY +
          projectLinearRange(
            pt.y,
            interactionState.yMin,
            interactionState.yMax,
            chartHeight,
            0,
          ) *
            interactionState.zoomState.scaleY;
        const dx = px - chartX;
        const dy = py - chartY;
        const dist = dx * dx + dy * dy;
        if (dist < bestDist) {
          bestDist = dist;
          bestSeries = series.label;
          bestPoint = pt;
        }
      }
    }

    return bestPoint && Math.sqrt(bestDist) < 20
      ? [`${bestSeries}: (${fmt(bestPoint.x)}, ${fmt(bestPoint.y)})`]
      : [];
  }

  if (data.type === "heatmap" && interactionState.type === "heatmap") {
    const col = clampIndex(
      Math.floor((chartX / chartWidth) * interactionState.xCategories.length),
      interactionState.xCategories.length,
    );
    const row = clampIndex(
      interactionState.yCategories.length -
        1 -
        Math.floor(
          (chartY / chartHeight) * interactionState.yCategories.length,
        ),
      interactionState.yCategories.length,
    );

    if (col < 0 || row < 0) return [];

    const lines = [
      `${data.xName}: ${fmt(interactionState.xCategories[col])}, ${data.yName}: ${fmt(interactionState.yCategories[row])}`,
    ];

    for (const series of data.series) {
      const cellValue = findHeatmapCellValue(series, col, row);
      if (cellValue !== null) {
        lines.push(`${series.label}: ${fmt(cellValue)}`);
      }
    }

    return lines.length > 1 ? lines : [];
  }

  return [];
}

/** Find the point with the nearest x value via linear scan (data may be unsorted). */
function findNearestX(
  series: import("@/shared/lib/chartTypes").ChartSeries,
  targetX: number,
) {
  if (series.pointCount === 0) return null;

  let best = 0;
  let bestDist = Math.abs(getXYPoint(series, 0).x - targetX);
  for (let i = 1; i < series.pointCount; i++) {
    const dist = Math.abs(getXYPoint(series, i).x - targetX);
    if (dist < bestDist) {
      bestDist = dist;
      best = i;
    }
  }
  return getXYPoint(series, best);
}

function findHeatmapCellValue(
  series: import("@/shared/lib/chartTypes").HeatmapSeries,
  col: number,
  row: number,
) {
  for (let i = 0; i < series.pointCount; i++) {
    const point = getXYZPoint(series, i);
    if (point.x === col && point.y === row) {
      return point.z;
    }
  }
  return null;
}

function fmt(n: number): string {
  return Number.isInteger(n) ? String(n) : n.toPrecision(6);
}

function clampIndex(index: number, length: number): number {
  if (length <= 0) return -1;
  return Math.min(Math.max(index, 0), length - 1);
}
