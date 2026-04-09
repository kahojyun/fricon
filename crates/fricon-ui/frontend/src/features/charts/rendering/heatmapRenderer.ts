/**
 * Heatmap renderer — draws a heatmap using instanced quads.
 * Each cell is an instance with position (col, row) and a normalized value
 * that maps to a 5-stop color ramp matching the previous ECharts palette.
 */

import type { ChartSeries } from "@/shared/lib/chartTypes";
import { createBuffer, createProgram, hexToRgb } from "./webgl";
import { heatmapFragmentSource, heatmapVertexSource } from "./shaders/heatmap";

export const COLOR_RAMP = [
  "#2c7bb6",
  "#abd9e9",
  "#ffffbf",
  "#fdae61",
  "#d7191c",
];

export interface HeatmapRenderState {
  program: WebGLProgram;
  cornerBuffer: WebGLBuffer;
  cellBuffer: WebGLBuffer;
  instanceCount: number;
  vao: WebGLVertexArrayObject;
  valueMin: number;
  valueMax: number;
  uMatrix: WebGLUniformLocation | null;
  uColorRamp: WebGLUniformLocation | null;
}

export function createHeatmapRenderState(
  gl: WebGL2RenderingContext,
): HeatmapRenderState {
  const program = createProgram(gl, heatmapVertexSource, heatmapFragmentSource);

  // Unit quad corners (two triangles)
  // prettier-ignore
  const corners = new Float32Array([
    0, 0,
    1, 0,
    1, 1,
    0, 0,
    1, 1,
    0, 1,
  ]);

  const vao = gl.createVertexArray();
  if (!vao) throw new Error("Failed to create vertex array");
  gl.bindVertexArray(vao);

  // Corner buffer (per-vertex)
  const cornerBuffer = createBuffer(gl, corners);
  const aCorner = gl.getAttribLocation(program, "a_corner");
  gl.enableVertexAttribArray(aCorner);
  gl.vertexAttribPointer(aCorner, 2, gl.FLOAT, false, 0, 0);

  // Cell buffer (per-instance)
  const cellBuffer = createBuffer(gl, new Float32Array(0), gl.DYNAMIC_DRAW);
  const aCell = gl.getAttribLocation(program, "a_cell");
  gl.enableVertexAttribArray(aCell);
  gl.vertexAttribPointer(aCell, 3, gl.FLOAT, false, 0, 0);
  gl.vertexAttribDivisor(aCell, 1); // per-instance

  gl.bindVertexArray(null);

  return {
    program,
    cornerBuffer,
    cellBuffer,
    instanceCount: 0,
    vao,
    valueMin: 0,
    valueMax: 1,
    uMatrix: gl.getUniformLocation(program, "u_matrix"),
    uColorRamp: gl.getUniformLocation(program, "u_colorRamp"),
  };
}

export function syncHeatmapRenderState(
  gl: WebGL2RenderingContext,
  state: HeatmapRenderState,
  series: ChartSeries[],
): void {
  const { valueMin, valueMax, instanceData } = buildHeatmapInstances(series);
  gl.bindBuffer(gl.ARRAY_BUFFER, state.cellBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, instanceData, gl.DYNAMIC_DRAW);
  state.instanceCount = instanceData.length / 3;
  state.valueMin = valueMin;
  state.valueMax = valueMax;
}

export function drawHeatmap(
  gl: WebGL2RenderingContext,
  state: HeatmapRenderState,
  numCols: number,
  numRows: number,
): void {
  const { program, vao, instanceCount, uMatrix, uColorRamp } = state;
  gl.useProgram(program);

  // Build matrix that maps grid coords (0..numCols, 0..numRows) → clip space
  const sx = numCols > 0 ? 2 / numCols : 1;
  const sy = numRows > 0 ? 2 / numRows : 1;
  // prettier-ignore
  const matrix = new Float32Array([
    sx, 0,  0,
    0,  sy, 0,
    -1, -1, 1,
  ]);

  gl.uniformMatrix3fv(uMatrix, false, matrix);

  // Upload color ramp
  const rampFlat = new Float32Array(15);
  for (let i = 0; i < 5; i++) {
    const [r, g, b] = hexToRgb(COLOR_RAMP[i]);
    rampFlat[i * 3] = r;
    rampFlat[i * 3 + 1] = g;
    rampFlat[i * 3 + 2] = b;
  }
  gl.uniform3fv(uColorRamp, rampFlat);

  gl.bindVertexArray(vao);
  gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, instanceCount);
  gl.bindVertexArray(null);
}

export function destroyHeatmapRenderState(
  gl: WebGL2RenderingContext,
  state: HeatmapRenderState,
): void {
  gl.deleteBuffer(state.cornerBuffer);
  gl.deleteBuffer(state.cellBuffer);
  gl.deleteVertexArray(state.vao);
  gl.deleteProgram(state.program);
}

function buildHeatmapInstances(series: ChartSeries[]): {
  valueMin: number;
  valueMax: number;
  instanceData: Float32Array;
} {
  let min = Infinity;
  let max = -Infinity;
  for (const s of series) {
    for (const value of s.data) {
      const cellValue = value[2];
      if (cellValue === undefined) continue;
      if (cellValue < min) min = cellValue;
      if (cellValue > max) max = cellValue;
    }
  }
  if (!isFinite(min)) min = 0;
  if (!isFinite(max)) max = 1;
  const range = max !== min ? max - min : 1;

  const instances: number[] = [];
  for (const s of series) {
    for (const point of s.data) {
      const value = point[2];
      if (value === undefined || !Number.isFinite(value)) continue;
      instances.push(point[0], point[1], (value - min) / range);
    }
  }

  return {
    valueMin: min,
    valueMax: max,
    instanceData: new Float32Array(instances),
  };
}
