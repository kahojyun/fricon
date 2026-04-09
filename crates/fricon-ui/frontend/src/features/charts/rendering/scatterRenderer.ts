/**
 * XY point renderer — draws multi-series scatter plots via GL_POINTS.
 * Consumes ChartOptions (type="xy") and renders into a WebGL2 context.
 */

import type { ChartOptions, ChartSeries } from "@/shared/lib/chartTypes";
import {
  createBuffer,
  createProgram,
  getLiveSeriesAppearance,
  getSeriesColor,
} from "./webgl";
import { scatterFragmentSource, scatterVertexSource } from "./shaders/scatter";

export interface ScatterRenderState {
  program: WebGLProgram;
  uMatrix: WebGLUniformLocation | null;
  uColor: WebGLUniformLocation | null;
  uPointSize: WebGLUniformLocation | null;
  aPosition: number;
  seriesBuffers: {
    buffer: WebGLBuffer;
    count: number;
    capacity: number;
    values: Float64Array;
  }[];
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
    const flat = series[i].values;
    const existing = state.seriesBuffers[i];
    if (existing) {
      gl.bindBuffer(gl.ARRAY_BUFFER, existing.buffer);
      if (
        flat.length >= existing.values.length &&
        hasPrefix(flat, existing.values)
      ) {
        if (flat.length > existing.capacity) {
          existing.capacity = Math.max(flat.length, existing.capacity * 2, 2);
          gl.bufferData(
            gl.ARRAY_BUFFER,
            new Float32Array(existing.capacity),
            gl.DYNAMIC_DRAW,
          );
          gl.bufferSubData(gl.ARRAY_BUFFER, 0, toFloat32Array(flat));
        } else if (flat.length > existing.values.length) {
          gl.bufferSubData(
            gl.ARRAY_BUFFER,
            existing.values.length * 4,
            toFloat32Array(flat.subarray(existing.values.length)),
          );
        }
      } else {
        if (flat.length > existing.capacity) {
          existing.capacity = flat.length;
          gl.bufferData(
            gl.ARRAY_BUFFER,
            new Float32Array(existing.capacity),
            gl.DYNAMIC_DRAW,
          );
        }
        gl.bufferSubData(gl.ARRAY_BUFFER, 0, toFloat32Array(flat));
      }
      existing.count = series[i].pointCount;
      existing.values = flat;
      continue;
    }

    state.seriesBuffers.push({
      buffer: createBuffer(gl, toFloat32Array(flat), gl.DYNAMIC_DRAW),
      count: series[i].pointCount,
      capacity: flat.length,
      values: flat,
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
  data: Extract<ChartOptions, { type: "xy" }>,
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
      const style = getLiveSeriesAppearance(data.series[i], data.series, i);
      opacity = style.opacity;
      color = style.color;
      pointSize = (style.isCurrent ? 6 : 4) * dpr;
    } else {
      color = getSeriesColor(i);
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
    for (let i = 0; i < s.values.length; i += 2) {
      const x = s.values[i];
      const y = s.values[i + 1];
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

function hasPrefix(values: Float64Array, prefix: Float64Array) {
  if (prefix.length > values.length) return false;
  for (let i = 0; i < prefix.length; i++) {
    if (values[i] !== prefix[i]) return false;
  }
  return true;
}

function toFloat32Array(values: Float64Array) {
  return Float32Array.from(values);
}
