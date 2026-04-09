/**
 * Line chart renderer — draws multi-series line charts via GL_LINE_STRIP.
 * Consumes ChartOptions (type="line") and renders into a WebGL2 context.
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
import { lineFragmentSource, lineVertexSource } from "./shaders/line";

export interface LineRenderState {
  program: WebGLProgram;
  uMatrix: WebGLUniformLocation | null;
  uColor: WebGLUniformLocation | null;
  aPosition: number;
  seriesBuffers: {
    buffer: WebGLBuffer;
    count: number;
    capacity: number;
    values: Float32Array;
  }[];
}

function liveSeriesStyle(
  index: number,
  total: number,
): { isNewest: boolean; opacity: number } {
  const isNewest = index === total - 1;
  const opacity = isNewest
    ? 1.0
    : 0.12 + (0.5 * index) / Math.max(total - 2, 1);
  return { isNewest, opacity };
}

export function createLineRenderState(
  gl: WebGL2RenderingContext,
): LineRenderState {
  const program = createProgram(gl, lineVertexSource, lineFragmentSource);
  return {
    program,
    uMatrix: gl.getUniformLocation(program, "u_matrix"),
    uColor: gl.getUniformLocation(program, "u_color"),
    aPosition: gl.getAttribLocation(program, "a_position"),
    seriesBuffers: [],
  };
}

export function syncLineRenderState(
  gl: WebGL2RenderingContext,
  state: LineRenderState,
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
          gl.bufferSubData(gl.ARRAY_BUFFER, 0, flat);
        } else if (flat.length > existing.values.length) {
          gl.bufferSubData(
            gl.ARRAY_BUFFER,
            existing.values.length * 4,
            flat.subarray(existing.values.length),
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
        gl.bufferSubData(gl.ARRAY_BUFFER, 0, flat);
      }
      existing.count = series[i].pointCount;
      existing.values = flat;
      continue;
    }

    state.seriesBuffers.push({
      buffer: createBuffer(gl, flat, gl.DYNAMIC_DRAW),
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

export function drawLines(
  gl: WebGL2RenderingContext,
  state: LineRenderState,
  matrix: Float32Array,
  data: Extract<ChartOptions, { type: "line" }>,
  liveMode: boolean,
): void {
  const { program, seriesBuffers, uMatrix, uColor, aPosition } = state;
  gl.useProgram(program);

  gl.uniformMatrix3fv(uMatrix, false, matrix);

  for (let i = 0; i < seriesBuffers.length; i++) {
    const { buffer, count } = seriesBuffers[i];
    if (count === 0) continue;

    let color: [number, number, number];
    let opacity: number;

    if (liveMode && data.series.length > 1) {
      const style = liveSeriesStyle(i, data.series.length);
      const hex = style.isNewest ? LIVE_NEWEST_COLOR : LIVE_OLD_COLOR;
      color = hexToRgb(hex);
      opacity = style.opacity;
    } else {
      color = hexToRgb(SERIES_COLORS[i % SERIES_COLORS.length]);
      opacity = 1.0;
    }

    gl.uniform4f(uColor, color[0], color[1], color[2], opacity);

    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.enableVertexAttribArray(aPosition);
    gl.vertexAttribPointer(aPosition, 2, gl.FLOAT, false, 0, 0);

    gl.drawArrays(gl.LINE_STRIP, 0, count);
  }
}

export function destroyLineRenderState(
  gl: WebGL2RenderingContext,
  state: LineRenderState,
): void {
  for (const { buffer } of state.seriesBuffers) {
    gl.deleteBuffer(buffer);
  }
  gl.deleteProgram(state.program);
}

/** Compute x/y data bounds across all series for line charts. */
export function lineDataBounds(series: ChartSeries[]): {
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
      const x = s.values[i]!;
      const y = s.values[i + 1]!;
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
  // Add 5% padding
  const xPad = (xMax - xMin) * 0.05 || 0.5;
  const yPad = (yMax - yMin) * 0.05 || 0.5;
  return {
    xMin: xMin - xPad,
    xMax: xMax + xPad,
    yMin: yMin - yPad,
    yMax: yMax + yPad,
  };
}

function hasPrefix(values: Float32Array, prefix: Float32Array) {
  if (prefix.length > values.length) return false;
  for (let i = 0; i < prefix.length; i++) {
    if (values[i] !== prefix[i]) return false;
  }
  return true;
}
