import { describe, expect, it, vi } from "vitest";
import type { ChartSeries } from "@/shared/lib/chartTypes";
import {
  syncHeatmapRenderState,
  type HeatmapRenderState,
} from "./heatmapRenderer";
import { lineDataBounds } from "./lineRenderer";

describe("lineDataBounds", () => {
  it("ignores non-finite points when computing chart bounds", () => {
    const series: ChartSeries[] = [
      {
        name: "signal",
        data: [
          [0, 1],
          [2, Infinity],
          [Infinity, 3],
          [4, 5],
        ],
      },
    ];

    expect(lineDataBounds(series)).toEqual({
      xMin: -0.2,
      xMax: 4.2,
      yMin: 0.8,
      yMax: 5.2,
    });
  });
});

describe("syncHeatmapRenderState", () => {
  it("uses only finite cell values for min/max normalization", () => {
    const bufferData = vi.fn();
    const gl = {
      ARRAY_BUFFER: 0x8892,
      DYNAMIC_DRAW: 0x88e8,
      bindBuffer: vi.fn(),
      bufferData,
    } as unknown as WebGL2RenderingContext;

    const state = {
      cellBuffer: {} as WebGLBuffer,
      instanceCount: 0,
      valueMin: 0,
      valueMax: 0,
    } as HeatmapRenderState;

    const series: ChartSeries[] = [
      {
        name: "heat",
        data: [
          [0, 0, 1],
          [1, 0, Infinity],
          [0, 1, 5],
        ],
      },
    ];

    syncHeatmapRenderState(gl, state, series);

    expect(state.valueMin).toBe(1);
    expect(state.valueMax).toBe(5);
    expect(state.instanceCount).toBe(2);
    expect(bufferData).toHaveBeenCalledWith(
      gl.ARRAY_BUFFER,
      new Float32Array([0, 0, 0, 0, 1, 1]),
      gl.DYNAMIC_DRAW,
    );
  });
});
