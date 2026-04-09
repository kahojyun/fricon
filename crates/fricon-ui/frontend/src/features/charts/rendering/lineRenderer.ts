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
  seriesBuffers: { buffer: WebGLBuffer; count: number }[];
}

function parseLiveAge(name: string): number | null {
  const marker = /\[(current|-\d+)\]$/.exec(name)?.[1] ?? null;
  if (!marker) return null;
  if (marker === "current") return 0;
  return Number.parseInt(marker.slice(1), 10);
}

function liveSeriesStyle(
  name: string,
  allNames: string[],
  fallbackIndex: number,
  total: number,
): { isNewest: boolean; opacity: number } {
  const age = parseLiveAge(name);
  if (age == null) {
    const isNewest = fallbackIndex === total - 1;
    const opacity = isNewest
      ? 1.0
      : 0.12 + (0.5 * fallbackIndex) / Math.max(total - 2, 1);
    return { isNewest, opacity };
  }

  if (age === 0) {
    return { isNewest: true, opacity: 1.0 };
  }

  const maxAge = Math.max(
    ...allNames.map((candidate) => parseLiveAge(candidate) ?? 0),
    age,
  );
  const opacity = 0.12 + (0.5 * (maxAge - age)) / Math.max(maxAge, 1);
  return { isNewest: false, opacity };
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

  const liveNames = liveMode ? data.series.map((s) => s.name) : [];

  for (let i = 0; i < seriesBuffers.length; i++) {
    const { buffer, count } = seriesBuffers[i];
    if (count === 0) continue;

    let color: [number, number, number];
    let opacity: number;

    if (liveMode && data.series.length > 1) {
      const style = liveSeriesStyle(
        data.series[i].name,
        liveNames,
        i,
        data.series.length,
      );
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

function flattenSeriesPoints(series: ChartSeries): Float32Array {
  const flat = new Float32Array(series.data.length * 2);
  for (let i = 0; i < series.data.length; i++) {
    flat[i * 2] = series.data[i][0]!;
    flat[i * 2 + 1] = series.data[i][1]!;
  }
  return flat;
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
