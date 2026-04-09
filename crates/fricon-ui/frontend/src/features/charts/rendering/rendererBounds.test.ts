import { describe, expect, it, vi } from "vitest";
import type { ChartSeries, HeatmapSeries } from "@/shared/lib/chartTypes";
import {
  drawLines,
  lineDataBounds,
  type LineRenderState,
} from "./lineRenderer";
import {
  syncHeatmapRenderState,
  type HeatmapRenderState,
} from "./heatmapRenderer";
import {
  drawScatter,
  scatterDataBounds,
  type ScatterRenderState,
} from "./scatterRenderer";
import { getSeriesColor } from "./webgl";

describe("lineDataBounds", () => {
  it("ignores non-finite points when computing chart bounds", () => {
    const series: ChartSeries[] = [
      xySeries("signal", "signal", [
        [0, 1],
        [2, Infinity],
        [Infinity, 3],
        [4, 5],
      ]),
    ];

    expect(lineDataBounds(series)).toEqual({
      xMin: -0.2,
      xMax: 4.2,
      yMin: 0.8,
      yMax: 5.2,
    });
  });

  it("keeps all current live-series variants highlighted together", () => {
    const uniform4f = vi.fn();
    const gl = {
      useProgram: vi.fn(),
      uniformMatrix3fv: vi.fn(),
      uniform4f,
      bindBuffer: vi.fn(),
      enableVertexAttribArray: vi.fn(),
      vertexAttribPointer: vi.fn(),
      drawArrays: vi.fn(),
      ARRAY_BUFFER: 0x8892,
      FLOAT: 0x1406,
      LINE_STRIP: 0x0003,
    } as unknown as WebGL2RenderingContext;

    const state = {
      program: {} as WebGLProgram,
      uMatrix: {} as WebGLUniformLocation,
      uColor: {} as WebGLUniformLocation,
      aPosition: 0,
      seriesBuffers: [
        {
          buffer: {} as WebGLBuffer,
          count: 1,
          capacity: 2,
          values: new Float64Array([0, 0]),
        },
        {
          buffer: {} as WebGLBuffer,
          count: 1,
          capacity: 2,
          values: new Float64Array([1, 1]),
        },
        {
          buffer: {} as WebGLBuffer,
          count: 1,
          capacity: 2,
          values: new Float64Array([2, 2]),
        },
      ],
    } satisfies LineRenderState;

    drawLines(
      gl,
      state,
      new Float32Array(9),
      {
        type: "xy",
        projection: "trend",
        drawStyle: "line",
        xName: "x",
        yName: null,
        series: [
          xySeries("row:4:signal:real", "signal (real)", [[0, 0]]),
          xySeries("row:5:signal:real", "signal (real)", [[1, 1]]),
          xySeries("row:5:signal:imag", "signal (imag)", [[2, 2]]),
        ],
      },
      true,
    );

    expect(uniform4f.mock.calls).toEqual([
      [
        state.uColor,
        getSeriesColor(0)[0] * 0.55,
        getSeriesColor(0)[1] * 0.55,
        getSeriesColor(0)[2] * 0.55,
        0.16,
      ],
      [state.uColor, ...getSeriesColor(0), 1],
      [state.uColor, ...getSeriesColor(1), 1],
    ]);
  });
});

describe("scatterDataBounds", () => {
  it("ignores non-finite points when computing chart bounds", () => {
    const series: ChartSeries[] = [
      xySeries("points", "points", [
        [0, 1],
        [2, Number.NaN],
        [Infinity, 3],
        [4, 5],
      ]),
    ];

    expect(scatterDataBounds(series)).toEqual({
      xMin: -0.2,
      xMax: 4.2,
      yMin: 0.8,
      yMax: 5.2,
    });
  });

  it("keeps all current live point variants highlighted together", () => {
    const uniform4f = vi.fn();
    const uniform1f = vi.fn();
    const gl = {
      useProgram: vi.fn(),
      uniformMatrix3fv: vi.fn(),
      uniform4f,
      uniform1f,
      bindBuffer: vi.fn(),
      enableVertexAttribArray: vi.fn(),
      vertexAttribPointer: vi.fn(),
      drawArrays: vi.fn(),
      ARRAY_BUFFER: 0x8892,
      FLOAT: 0x1406,
      POINTS: 0x0000,
    } as unknown as WebGL2RenderingContext;

    const state = {
      program: {} as WebGLProgram,
      uMatrix: {} as WebGLUniformLocation,
      uColor: {} as WebGLUniformLocation,
      uPointSize: {} as WebGLUniformLocation,
      aPosition: 0,
      seriesBuffers: [
        {
          buffer: {} as WebGLBuffer,
          count: 1,
          capacity: 2,
          values: new Float64Array([0, 0]),
        },
        {
          buffer: {} as WebGLBuffer,
          count: 1,
          capacity: 2,
          values: new Float64Array([1, 1]),
        },
        {
          buffer: {} as WebGLBuffer,
          count: 1,
          capacity: 2,
          values: new Float64Array([2, 2]),
        },
      ],
    } satisfies ScatterRenderState;

    drawScatter(
      gl,
      state,
      new Float32Array(9),
      {
        type: "xy",
        projection: "trend",
        drawStyle: "line_points",
        xName: "x",
        yName: null,
        series: [
          xySeries("group:4:signal:real", "signal (real)", [[0, 0]]),
          xySeries("group:5:signal:real", "signal (real)", [[1, 1]]),
          xySeries("group:5:signal:imag", "signal (imag)", [[2, 2]]),
        ],
      },
      true,
    );

    expect(uniform4f.mock.calls).toEqual([
      [
        state.uColor,
        getSeriesColor(0)[0] * 0.55,
        getSeriesColor(0)[1] * 0.55,
        getSeriesColor(0)[2] * 0.55,
        0.16,
      ],
      [state.uColor, ...getSeriesColor(0), 1],
      [state.uColor, ...getSeriesColor(1), 1],
    ]);
    const pointSizes = (uniform1f.mock.calls as [unknown, number][]).map(
      ([, size]) => size,
    );
    expect(pointSizes).toEqual([4, 6, 6]);
  });
});

describe("syncHeatmapRenderState", () => {
  it("uses only finite cell values for min/max normalization", () => {
    const bufferData = vi.fn();
    const bufferSubData = vi.fn();
    const gl = {
      ARRAY_BUFFER: 0x8892,
      DYNAMIC_DRAW: 0x88e8,
      bindBuffer: vi.fn(),
      bufferData,
      bufferSubData,
    } as unknown as WebGL2RenderingContext;

    const state = {
      cellBuffer: {} as WebGLBuffer,
      instanceCount: 0,
      capacity: 0,
      instanceData: new Float64Array(0),
      valueMin: 0,
      valueMax: 0,
    } as unknown as HeatmapRenderState;

    const series: HeatmapSeries[] = [
      xyzSeries("heat", "heat", [
        [0, 0, 1],
        [1, 0, Infinity],
        [0, 1, 5],
      ]),
    ];

    syncHeatmapRenderState(gl, state, series);

    expect(state.valueMin).toBe(1);
    expect(state.valueMax).toBe(5);
    expect(state.instanceCount).toBe(2);
    expect(bufferData).toHaveBeenCalledWith(
      gl.ARRAY_BUFFER,
      new Float32Array(6),
      gl.DYNAMIC_DRAW,
    );
    expect(bufferSubData).toHaveBeenCalledWith(
      gl.ARRAY_BUFFER,
      0,
      new Float32Array([0, 0, 0, 0, 1, 1]),
    );
  });
});

function xySeries(
  id: string,
  label: string,
  points: [number, number][],
): ChartSeries {
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
): HeatmapSeries {
  return {
    id,
    label,
    pointCount: points.length,
    values: Float64Array.from(points.flat()),
  };
}
