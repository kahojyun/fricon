import { describe, expect, it, vi } from "vitest";
import type { ChartSeries, HeatmapSeries } from "@/shared/lib/chartTypes";
import {
  drawLines,
  lineDataBounds,
  type LineRenderState,
} from "./lineRenderer";
import {
  buildHeatmapGeometry,
  heatmapDataBounds,
  EMPTY_HEATMAP_GEOMETRY,
} from "./heatmapGeometry";
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
        plotMode: "quantity_vs_sweep",
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
        plotMode: "quantity_vs_sweep",
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
  it("builds midpoint-derived numeric cell geometry", () => {
    const geometry = buildHeatmapGeometry([
      xyzSeries("heat", "heat", [
        [10, 5, 1],
        [20, 5, 2],
        [40, 9, 3],
      ]),
    ]);

    expect(geometry).toEqual({
      xMin: 5,
      xMax: 50,
      yMin: 3,
      yMax: 11,
      series: [
        {
          seriesId: "heat",
          cells: [
            { x: 10, y: 5, z: 1, x0: 5, x1: 15, y0: 3, y1: 7 },
            { x: 20, y: 5, z: 2, x0: 15, x1: 30, y0: 3, y1: 7 },
            { x: 40, y: 9, z: 3, x0: 30, x1: 50, y0: 7, y1: 11 },
          ],
        },
      ],
    });
  });

  it("uses a default half-step for singleton axes", () => {
    expect(
      buildHeatmapGeometry([xyzSeries("heat", "heat", [[7, 11, 5]])]),
    ).toEqual({
      xMin: 6.5,
      xMax: 7.5,
      yMin: 10.5,
      yMax: 11.5,
      series: [
        {
          seriesId: "heat",
          cells: [{ x: 7, y: 11, z: 5, x0: 6.5, x1: 7.5, y0: 10.5, y1: 11.5 }],
        },
      ],
    });
  });

  it("returns default bounds when there are no finite heatmap cells", () => {
    expect(
      heatmapDataBounds([xyzSeries("heat", "heat", [[0, 0, Number.NaN]])]),
    ).toEqual({
      xMin: EMPTY_HEATMAP_GEOMETRY.xMin,
      xMax: EMPTY_HEATMAP_GEOMETRY.xMax,
      yMin: EMPTY_HEATMAP_GEOMETRY.yMin,
      yMax: EMPTY_HEATMAP_GEOMETRY.yMax,
    });
  });

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
      geometry: EMPTY_HEATMAP_GEOMETRY,
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
    expect(state.geometry).toEqual({
      xMin: -0.5,
      xMax: 0.5,
      yMin: -0.5,
      yMax: 1.5,
      series: [
        {
          seriesId: "heat",
          cells: [
            { x: 0, y: 0, z: 1, x0: -0.5, x1: 0.5, y0: -0.5, y1: 0.5 },
            { x: 0, y: 1, z: 5, x0: -0.5, x1: 0.5, y0: 0.5, y1: 1.5 },
          ],
        },
      ],
    });
    expect(bufferData).toHaveBeenCalledWith(
      gl.ARRAY_BUFFER,
      new Float32Array(10),
      gl.DYNAMIC_DRAW,
    );
    expect(bufferSubData).toHaveBeenCalledWith(
      gl.ARRAY_BUFFER,
      0,
      new Float32Array([-0.5, -0.5, 0.5, 0.5, 0, -0.5, 0.5, 0.5, 1.5, 1]),
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
