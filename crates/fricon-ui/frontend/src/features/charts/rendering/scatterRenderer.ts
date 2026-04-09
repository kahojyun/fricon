/**
 * Scatter chart renderer — draws multi-series scatter plots via GL_POINTS.
 * Consumes ChartOptions (type="scatter") and renders into a WebGL2 context.
 */

import type { ChartOptions, ChartSeries } from "@/shared/lib/chartTypes";
import {
  createBuffer,
  createProgram,
  hexToRgb,
  LIVE_NEWEST_COLOR,
  LIVE_OLD_COLOR,
  SERIES_COLORS,
} from "./webgl";
import { scatterFragmentSource, scatterVertexSource } from "./shaders/scatter";

export interface ScatterRenderState {
  program: WebGLProgram;
  uMatrix: WebGLUniformLocation | null;
  uColor: WebGLUniformLocation | null;
  uPointSize: WebGLUniformLocation | null;
  aPosition: number;
  seriesBuffers: { buffer: WebGLBuffer; count: number }[];
}

export function createScatterRenderState(
  gl: WebGL2RenderingContext,
): ScatterRenderState {
  const program = createProgram(gl, scatterVertexSource, scatterFragmentSource);
  return {
    program,
    uMatrix: gl.getUniformLocation(program, "u_matrix"),
    uColor: gl.getUniformLocation(program, "u_color"),
    uPointSize: gl.getUniformLocation(program, "u_pointSize"),
    aPosition: gl.getAttribLocation(program, "a_position"),
    seriesBuffers: [],
  };
}

export function syncScatterRenderState(
  gl: WebGL2RenderingContext,
  state: ScatterRenderState,
  series: ChartSeries[],
): void {
  for (let i = 0; i < series.length; i++) {
    const flat = flattenSeriesPoints(series[i]);
    const existing = state.seriesBuffers[i];
    if (existing) {
      gl.bindBuffer(gl.ARRAY_BUFFER, existing.buffer);
      gl.bufferData(gl.ARRAY_BUFFER, flat, gl.DYNAMIC_DRAW);
      existing.count = series[i].data.length;
      continue;
    }

    state.seriesBuffers.push({
      buffer: createBuffer(gl, flat, gl.DYNAMIC_DRAW),
      count: series[i].data.length,
    });
  }

  while (state.seriesBuffers.length > series.length) {
    const removed = state.seriesBuffers.pop();
    if (removed) gl.deleteBuffer(removed.buffer);
  }
}

export function drawScatter(
  gl: WebGL2RenderingContext,
  state: ScatterRenderState,
  matrix: Float32Array,
  data: Extract<ChartOptions, { type: "scatter" }>,
  liveMode: boolean,
): void {
  const { program, seriesBuffers, uMatrix, uColor, uPointSize, aPosition } =
    state;
  gl.useProgram(program);

  gl.uniformMatrix3fv(uMatrix, false, matrix);

  const dpr = window.devicePixelRatio || 1;

  for (let i = 0; i < seriesBuffers.length; i++) {
    const { buffer, count } = seriesBuffers[i];
    if (count === 0) continue;

    let color: [number, number, number];
    let opacity: number;
    let pointSize: number;

    if (liveMode && data.series.length > 1) {
      const total = data.series.length;
      const isNewest = i === total - 1;
      opacity = isNewest ? 1.0 : 0.12 + (0.5 * i) / Math.max(total - 2, 1);
      const hex = isNewest ? LIVE_NEWEST_COLOR : LIVE_OLD_COLOR;
      color = hexToRgb(hex);
      pointSize = (isNewest ? 6 : 4) * dpr;
    } else {
      color = hexToRgb(SERIES_COLORS[i % SERIES_COLORS.length]);
      opacity = 1.0;
      pointSize = 6 * dpr;
    }

    gl.uniform4f(uColor, color[0], color[1], color[2], opacity);
    gl.uniform1f(uPointSize, pointSize);

    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.enableVertexAttribArray(aPosition);
    gl.vertexAttribPointer(aPosition, 2, gl.FLOAT, false, 0, 0);

    gl.drawArrays(gl.POINTS, 0, count);
  }
}

function flattenSeriesPoints(series: ChartSeries): Float32Array {
  const flat = new Float32Array(series.data.length * 2);
  for (let i = 0; i < series.data.length; i++) {
    flat[i * 2] = series.data[i][0]!;
    flat[i * 2 + 1] = series.data[i][1]!;
  }
  return flat;
}

export function destroyScatterRenderState(
  gl: WebGL2RenderingContext,
  state: ScatterRenderState,
): void {
  for (const { buffer } of state.seriesBuffers) {
    gl.deleteBuffer(buffer);
  }
  gl.deleteProgram(state.program);
}

/** Compute x/y data bounds across all series for scatter charts. */
export function scatterDataBounds(series: ChartSeries[]): {
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
} {
  let xMin = Infinity,
    xMax = -Infinity,
    yMin = Infinity,
    yMax = -Infinity;
  for (const s of series) {
    for (const d of s.data) {
      const x = d[0];
      const y = d[1];
      if (!Number.isFinite(x) || !Number.isFinite(y)) continue;
      if (x < xMin) xMin = x;
      if (x > xMax) xMax = x;
      if (y < yMin) yMin = y;
      if (y > yMax) yMax = y;
    }
  }
  if (!isFinite(xMin)) {
    xMin = 0;
    xMax = 1;
  }
  if (!isFinite(yMin)) {
    yMin = 0;
    yMax = 1;
  }
  const xPad = (xMax - xMin) * 0.05 || 0.5;
  const yPad = (yMax - yMin) * 0.05 || 0.5;
  return {
    xMin: xMin - xPad,
    xMax: xMax + xPad,
    yMin: yMin - yPad,
    yMax: yMax + yPad,
  };
}
