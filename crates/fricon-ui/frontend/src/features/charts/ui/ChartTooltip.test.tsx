import { describe, expect, it } from "vitest";
import type { ChartOptions } from "@/shared/lib/chartTypes";
import type { ChartInteractionState } from "../hooks/useWebGLChart";
import { getTooltipLines } from "./tooltipLines";

const margin = {
  top: 20,
  right: 20,
  bottom: 40,
  left: 60,
};

describe("getTooltipLines", () => {
  it("maps line hover positions through the zoom transform", () => {
    const data: ChartOptions = {
      type: "xy",
      plotMode: "quantity_vs_sweep",
      drawStyle: "line",
      xName: "x",
      yName: null,
      series: [
        xySeries("series", "series", [
          [0, 0],
          [5, 5],
          [10, 10],
        ]),
      ],
    };

    const interactionState: ChartInteractionState = {
      type: "xy",
      xMin: 0,
      xMax: 10,
      yMin: 0,
      yMax: 10,
      margin,
      zoomState: {
        scaleX: 2,
        scaleY: 1,
        translateX: -50,
        translateY: 0,
      },
    };

    expect(getTooltipLines(data, interactionState, 50, 50, 100, 100)).toEqual([
      "series: (5, 5)",
    ]);
  });

  it("matches scatter points in zoomed screen space", () => {
    const data: ChartOptions = {
      type: "xy",
      plotMode: "xy",
      drawStyle: "points",
      xName: "x",
      yName: "y",
      series: [xySeries("series", "series", [[5, 5]])],
    };

    const interactionState: ChartInteractionState = {
      type: "xy",
      xMin: 0,
      xMax: 10,
      yMin: 0,
      yMax: 10,
      margin,
      zoomState: {
        scaleX: 2,
        scaleY: 2,
        translateX: -50,
        translateY: -50,
      },
    };

    expect(getTooltipLines(data, interactionState, 50, 50, 100, 100)).toEqual([
      "series: (5, 5)",
    ]);
  });

  it("returns heatmap categories and cell values", () => {
    const data: ChartOptions = {
      type: "heatmap",
      xName: "x",
      yName: "y",
      xCategories: [1, 2],
      yCategories: [10, 20],
      series: [xyzSeries("z", "z", [[1, 1, 200]])],
    };

    const interactionState: ChartInteractionState = {
      type: "heatmap",
      xCategories: [1, 2],
      yCategories: [10, 20],
      margin,
    };

    expect(getTooltipLines(data, interactionState, 75, 25, 100, 100)).toEqual([
      "x: 2, y: 20",
      "z: 200",
    ]);
  });
});

function xySeries(id: string, label: string, points: [number, number][]) {
  return {
    id,
    label,
    pointCount: points.length,
    values: Float64Array.from(points.flat()),
  };
}

function xyzSeries(
  id: string,
  label: string,
  points: [number, number, number][],
) {
  return {
    id,
    label,
    pointCount: points.length,
    values: Float64Array.from(points.flat()),
  };
}
