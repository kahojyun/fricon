/**
 * Heatmap renderer — draws a heatmap using instanced quads.
 * Each cell is an instance with position (col, row) and a normalized value
 * that maps to a 5-stop color ramp matching the previous ECharts palette.
 */

import type { ChartSeries } from "@/shared/lib/chartTypes";
import { createProgram, hexToRgb } from "./webgl";
import { heatmapFragmentSource, heatmapVertexSource } from "./shaders/heatmap";

const COLOR_RAMP = ["#2c7bb6", "#abd9e9", "#ffffbf", "#fdae61", "#d7191c"];

export interface HeatmapRenderState {
  program: WebGLProgram;
  cornerBuffer: WebGLBuffer;
  cellBuffer: WebGLBuffer;
  instanceCount: number;
  vao: WebGLVertexArrayObject;
}

export function createHeatmapRenderState(
  gl: WebGL2RenderingContext,
  series: ChartSeries[],
): HeatmapRenderState {
  const program = createProgram(gl, heatmapVertexSource, heatmapFragmentSource);

  // Compute value range for normalization
  let min = Infinity;
  let max = -Infinity;
  for (const s of series) {
    for (const v of s.data) {
      const val = v[2];
      if (val === undefined) continue;
      if (val < min) min = val;
      if (val > max) max = val;
    }
  }
  if (!isFinite(min)) min = 0;
  if (!isFinite(max)) max = 1;
  const range = max !== min ? max - min : 1;

  // Build per-instance data: (col, row, normalizedValue)
  const instances: number[] = [];
  for (const s of series) {
    for (const d of s.data) {
      const col = d[0];
      const row = d[1];
      const val = d[2];
      if (val === undefined || !Number.isFinite(val)) continue;
      instances.push(col, row, (val - min) / range);
    }
  }
  const instanceCount = instances.length / 3;

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
  gl.bindVertexArray(vao);

  // Corner buffer (per-vertex)
  const cornerBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, cornerBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, corners, gl.STATIC_DRAW);
  const aCorner = gl.getAttribLocation(program, "a_corner");
  gl.enableVertexAttribArray(aCorner);
  gl.vertexAttribPointer(aCorner, 2, gl.FLOAT, false, 0, 0);

  // Cell buffer (per-instance)
  const cellBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, cellBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(instances), gl.STATIC_DRAW);
  const aCell = gl.getAttribLocation(program, "a_cell");
  gl.enableVertexAttribArray(aCell);
  gl.vertexAttribPointer(aCell, 3, gl.FLOAT, false, 0, 0);
  gl.vertexAttribDivisor(aCell, 1); // per-instance

  gl.bindVertexArray(null);

  return { program, cornerBuffer, cellBuffer, instanceCount, vao };
}

export function drawHeatmap(
  gl: WebGL2RenderingContext,
  state: HeatmapRenderState,
  numCols: number,
  numRows: number,
): void {
  const { program, vao, instanceCount } = state;
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

  const uMatrix = gl.getUniformLocation(program, "u_matrix");
  gl.uniformMatrix3fv(uMatrix, false, matrix);

  // Upload color ramp
  const uColorRamp = gl.getUniformLocation(program, "u_colorRamp");
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
