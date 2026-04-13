import {
  getXYPoint,
  type ChartOptions,
  type NumericLabelFormatOptions,
} from "@/shared/lib/chartTypes";
import type { ChartInteractionState } from "../hooks/useWebGLChart";
import {
  invertZoomedLinearRange,
  projectLinearRange,
} from "../rendering/mathUtils";
import { formatNumericLabel } from "../rendering/numericLabelFormat";

export function getTooltipLines(
  data: ChartOptions,
  numericLabelFormat: NumericLabelFormatOptions,
  interactionState: ChartInteractionState,
  chartX: number,
  chartY: number,
  chartWidth: number,
  chartHeight: number,
): string[] {
  if (data.type === "xy" && interactionState.type === "xy") {
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
      const nearest =
        data.plotMode === "quantity_vs_sweep" && data.drawStyle !== "points"
          ? findNearestX(series, dataX)
          : findNearestPoint(
              series,
              interactionState,
              chartX,
              chartY,
              chartWidth,
              chartHeight,
            );
      return nearest
        ? [
            `${series.label}: (${formatNumericLabel(nearest.x, numericLabelFormat)}, ${formatNumericLabel(nearest.y, numericLabelFormat)})`,
          ]
        : [];
    });
  }

  if (data.type === "heatmap" && interactionState.type === "heatmap") {
    const dataX = invertZoomedLinearRange(
      chartX,
      interactionState.zoomState.translateX,
      interactionState.zoomState.scaleX,
      interactionState.xMin,
      interactionState.xMax,
      0,
      chartWidth,
    );
    const dataY = invertZoomedLinearRange(
      chartY,
      interactionState.zoomState.translateY,
      interactionState.zoomState.scaleY,
      interactionState.yMin,
      interactionState.yMax,
      chartHeight,
      0,
    );
    const lines: string[] = [];
    let hoveredCell: { x: number; y: number } | null = null;

    for (const series of data.series) {
      const cellValue = findHeatmapCellValue(
        interactionState,
        series.id,
        dataX,
        dataY,
      );
      if (!cellValue) continue;
      hoveredCell ??= { x: cellValue.x, y: cellValue.y };
      lines.push(
        `${series.label}: ${formatNumericLabel(cellValue.z, numericLabelFormat)}`,
      );
    }

    if (!hoveredCell || lines.length === 0) return [];

    return [
      `${data.xName}: ${formatNumericLabel(hoveredCell.x, numericLabelFormat)}, ${data.yName}: ${formatNumericLabel(hoveredCell.y, numericLabelFormat)}`,
      ...lines,
    ];
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

function findNearestPoint(
  series: import("@/shared/lib/chartTypes").ChartSeries,
  interactionState: Extract<ChartInteractionState, { type: "xy" }>,
  chartX: number,
  chartY: number,
  chartWidth: number,
  chartHeight: number,
) {
  let bestDist = Infinity;
  let bestPoint: { x: number; y: number } | null = null;

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
      bestPoint = pt;
    }
  }

  return bestPoint && Math.sqrt(bestDist) < 20 ? bestPoint : null;
}

function findHeatmapCellValue(
  interactionState: Extract<ChartInteractionState, { type: "heatmap" }>,
  seriesId: string,
  dataX: number,
  dataY: number,
) {
  const geometry = interactionState.geometry.series.find(
    (item) => item.seriesId === seriesId,
  );
  if (!geometry) return null;

  for (const cell of geometry.cells) {
    if (
      dataX >= cell.x0 &&
      dataX <= cell.x1 &&
      dataY >= cell.y0 &&
      dataY <= cell.y1
    ) {
      return cell;
    }
  }
  return null;
}
