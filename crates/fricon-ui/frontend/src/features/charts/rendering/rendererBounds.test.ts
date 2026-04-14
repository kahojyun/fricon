import { describe, expect, it, vi } from "vitest";
import type { ChartSeries, HeatmapSeries } from "@/shared/lib/chartTypes";
import {
  drawLines,
  lineDataBounds,
  type LineRenderState,
} from "./lineRenderer";
import {
  deriveHeatmapLayout,
  EMPTY_HEATMAP_GEOMETRY,
  getHeatmapXTickValues,
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
    const { geometry } = deriveHeatmapLayout([
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
            { x: 20, y: 5, z: 2, x0: 15, x1: 25, y0: 3, y1: 7 },
            { x: 40, y: 9, z: 3, x0: 30, x1: 50, y0: 7, y1: 11 },
          ],
        },
      ],
    });
  });

  it("uses a default half-step for singleton axes", () => {
    expect(
      deriveHeatmapLayout([xyzSeries("heat", "heat", [[7, 11, 5]])]).geometry,
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

  it("keeps shared x centers for shared-grid heatmaps", () => {
    const layout = deriveHeatmapLayout([
      xyzSeries("heat", "heat", [
        [46, 120, 1],
        [7, 120, 2],
        [19, 120, 3],
        [46, 185, 4],
        [7, 185, 5],
        [19, 185, 6],
      ]),
    ]);

    expect(layout.xTopology).toBe("shared_grid");
    expect(layout.centers).toEqual({
      xValues: [7, 19, 46],
      yValues: [120, 185],
    });
    expect(getHeatmapXTickValues(layout.centers, layout.xTopology)).toEqual([
      7, 19, 46,
    ]);
  });

  it("keeps shared x centers for regular shared-grid heatmaps", () => {
    const layout = deriveHeatmapLayout([
      xyzSeries("heat", "heat", [
        [7, 120, 1],
        [19, 120, 2],
        [7, 185, 3],
        [19, 185, 4],
      ]),
    ]);

    expect(layout.xTopology).toBe("shared_grid");
    expect(layout.centers).toEqual({
      xValues: [7, 19],
      yValues: [120, 185],
    });
    expect(getHeatmapXTickValues(layout.centers, layout.xTopology)).toEqual([
      7, 19,
    ]);
  });

  it("classifies row-local x grids and suppresses explicit x ticks", () => {
    const layout = deriveHeatmapLayout([
      xyzSeries("heat", "heat", [
        [0, 0, 1],
        [10, 0, 2],
        [10, 1, 3],
        [30, 1, 4],
      ]),
    ]);

    expect(layout.xTopology).toBe("row_local_grid");
    expect(layout.centers).toEqual({
      xValues: [],
      yValues: [0, 1],
    });
    expect(
      getHeatmapXTickValues(layout.centers, layout.xTopology),
    ).toBeUndefined();
  });

  it("uses axis-span-aware tolerance for very small x ranges", () => {
    const layout = deriveHeatmapLayout([
      xyzSeries("heat", "heat", [
        [0, 0, 1],
        [1e-10, 0, 2],
        [1e-10, 1, 3],
        [3e-10, 1, 4],
      ]),
    ]);

    expect(layout.xTopology).toBe("row_local_grid");
    expect(layout.centers.xValues).toEqual([]);
  });

  it("uses axis-span-aware tolerance for large x ranges with tiny noise", () => {
    const layout = deriveHeatmapLayout([
      xyzSeries("heat", "heat", [
        [1e9, 0, 1],
        [2e9, 0, 2],
        [1e9 + 1e-6, 1, 3],
        [2e9 + 1e-6, 1, 4],
      ]),
    ]);

    expect(layout.xTopology).toBe("shared_grid");
    expect(layout.centers.xValues).toEqual([1e9, 2e9]);
  });

  it("uses row-local x spans when rows use different steps", () => {
    expect(
      deriveHeatmapLayout([
        xyzSeries("heat", "heat", [
          [0, 0, 1],
          [10, 0, 2],
          [10, 1, 3],
          [30, 1, 4],
        ]),
      ]).geometry,
    ).toEqual({
      xMin: -5,
      xMax: 40,
      yMin: -0.5,
      yMax: 1.5,
      series: [
        {
          seriesId: "heat",
          cells: [
            { x: 0, y: 0, z: 1, x0: -5, x1: 5, y0: -0.5, y1: 0.5 },
            { x: 10, y: 0, z: 2, x0: 5, x1: 15, y0: -0.5, y1: 0.5 },
            { x: 10, y: 1, z: 3, x0: 0, x1: 20, y0: 0.5, y1: 1.5 },
            { x: 30, y: 1, z: 4, x0: 20, x1: 40, y0: 0.5, y1: 1.5 },
          ],
        },
      ],
    });
  });

  it("uses row-local midpoint spans for non-shared x rows", () => {
    const layout = deriveHeatmapLayout([
      xyzSeries("heat", "heat", [
        [0, 0, 1],
        [100, 0, 2],
        [10, 1, 3],
        [30, 1, 4],
        [35, 1, 5],
      ]),
    ]);

    expect(layout.xTopology).toBe("row_local_grid");
    expect(layout.geometry).toEqual({
      xMin: -50,
      xMax: 150,
      yMin: -0.5,
      yMax: 1.5,
      series: [
        {
          seriesId: "heat",
          cells: [
            { x: 0, y: 0, z: 1, x0: -50, x1: 50, y0: -0.5, y1: 0.5 },
            { x: 100, y: 0, z: 2, x0: 50, x1: 150, y0: -0.5, y1: 0.5 },
            { x: 10, y: 1, z: 3, x0: 0, x1: 20, y0: 0.5, y1: 1.5 },
            { x: 30, y: 1, z: 4, x0: 20, x1: 32.5, y0: 0.5, y1: 1.5 },
            { x: 35, y: 1, z: 5, x0: 32.5, x1: 37.5, y0: 0.5, y1: 1.5 },
          ],
        },
      ],
    });
  });

  it("uses the nearest chart-wide spacing for singleton rows", () => {
    expect(
      deriveHeatmapLayout([
        xyzSeries("heat", "heat", [
          [0, 0, 1],
          [100, 0, 2],
          [10, 1, 3],
        ]),
      ]).geometry,
    ).toEqual({
      xMin: -50,
      xMax: 150,
      yMin: -0.5,
      yMax: 1.5,
      series: [
        {
          seriesId: "heat",
          cells: [
            { x: 0, y: 0, z: 1, x0: -50, x1: 50, y0: -0.5, y1: 0.5 },
            { x: 100, y: 0, z: 2, x0: 50, x1: 150, y0: -0.5, y1: 0.5 },
            { x: 10, y: 1, z: 3, x0: 5, x1: 15, y0: 0.5, y1: 1.5 },
          ],
        },
      ],
    });
  });

  it("preserves heatmap bounds for coordinates whose values are all NaN", () => {
    expect(
      deriveHeatmapLayout([xyzSeries("heat", "heat", [[0, 0, Number.NaN]])])
        .bounds,
    ).toEqual({
      xMin: -0.5,
      xMax: 0.5,
      yMin: -0.5,
      yMax: 0.5,
    });
  });

  it("preserves NaN-only rows and columns in axis centers", () => {
    const layout = deriveHeatmapLayout([
      xyzSeries("heat", "heat", [
        [0, 0, 1],
        [1, 0, Number.NaN],
        [0, 2, 3],
      ]),
    ]);

    expect(layout.xTopology).toBe("row_local_grid");
    expect(layout.centers).toEqual({
      xValues: [],
      yValues: [0, 2],
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
      bounds: {
        xMin: 0,
        xMax: 1,
        yMin: 0,
        yMax: 1,
      },
      centers: {
        xValues: [],
        yValues: [],
      },
      xTopology: "shared_grid",
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
      xMax: 1.5,
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
    expect(state.bounds).toEqual({
      xMin: -0.5,
      xMax: 1.5,
      yMin: -0.5,
      yMax: 1.5,
    });
    expect(state.centers).toEqual({
      xValues: [],
      yValues: [0, 1],
    });
    expect(state.xTopology).toBe("row_local_grid");
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
