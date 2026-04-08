import type { ChartOptions } from "@/shared/lib/chartTypes";
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
      const nearest = findNearestX(series.data, dataX);
      return nearest
        ? [`${series.name}: (${fmt(nearest[0])}, ${fmt(nearest[1])})`]
        : [];
    });
  }

  if (data.type === "scatter" && interactionState.type === "scatter") {
    let bestDist = Infinity;
    let bestSeries = "";
    let bestPoint: number[] | null = null;

    for (const series of data.series) {
      for (const pt of series.data) {
        const px =
          interactionState.zoomState.translateX +
          projectLinearRange(
            pt[0],
            interactionState.xMin,
            interactionState.xMax,
            0,
            chartWidth,
          ) *
            interactionState.zoomState.scaleX;
        const py =
          interactionState.zoomState.translateY +
          projectLinearRange(
            pt[1],
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
          bestSeries = series.name;
          bestPoint = pt;
        }
      }
    }

    return bestPoint && Math.sqrt(bestDist) < 20
      ? [`${bestSeries}: (${fmt(bestPoint[0])}, ${fmt(bestPoint[1])})`]
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
      const cell = series.data.find(
        (point) => point[0] === col && point[1] === row,
      );
      if (cell?.[2] !== undefined) {
        lines.push(`${series.name}: ${fmt(cell[2])}`);
      }
    }

    return lines.length > 1 ? lines : [];
  }

  return [];
}

/** Find the point with the nearest x value via linear scan (data may be unsorted). */
function findNearestX(data: number[][], targetX: number): number[] | null {
  if (data.length === 0) return null;

  let best = 0;
  let bestDist = Math.abs(data[0][0] - targetX);
  for (let i = 1; i < data.length; i++) {
    const dist = Math.abs(data[i][0] - targetX);
    if (dist < bestDist) {
      bestDist = dist;
      best = i;
    }
  }
  return data[best];
}

function fmt(n: number): string {
  return Number.isInteger(n) ? String(n) : n.toPrecision(6);
}

function clampIndex(index: number, length: number): number {
  if (length <= 0) return -1;
  return Math.min(Math.max(index, 0), length - 1);
}
