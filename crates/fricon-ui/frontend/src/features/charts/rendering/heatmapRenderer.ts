/**
 * Heatmap renderer — draws a heatmap using instanced quads.
 * Each cell is an instance with numeric bounds and a normalized value that
 * maps to a 5-stop color ramp matching the previous ECharts palette.
 */

import type { HeatmapSeries } from "@/shared/lib/chartTypes";
import { createBuffer, createProgram, hexToRgb } from "./webgl";
import {
  buildHeatmapGeometry,
  EMPTY_HEATMAP_GEOMETRY,
  type HeatmapGeometry,
} from "./heatmapGeometry";
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
  capacity: number;
  instanceData: Float64Array;
  geometry: HeatmapGeometry;
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
  const aRect = gl.getAttribLocation(program, "a_rect");
  gl.enableVertexAttribArray(aRect);
  gl.vertexAttribPointer(aRect, 4, gl.FLOAT, false, 20, 0);
  gl.vertexAttribDivisor(aRect, 1); // per-instance

  const aValue = gl.getAttribLocation(program, "a_value");
  gl.enableVertexAttribArray(aValue);
  gl.vertexAttribPointer(aValue, 1, gl.FLOAT, false, 20, 16);
  gl.vertexAttribDivisor(aValue, 1); // per-instance

  gl.bindVertexArray(null);

  return {
    program,
    cornerBuffer,
    cellBuffer,
    instanceCount: 0,
    capacity: 0,
    instanceData: new Float64Array(0),
    geometry: EMPTY_HEATMAP_GEOMETRY,
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
  series: HeatmapSeries[],
): void {
  const geometry = buildHeatmapGeometry(series);
  const { valueMin, valueMax, instanceData } = buildHeatmapInstances(geometry);
  gl.bindBuffer(gl.ARRAY_BUFFER, state.cellBuffer);
  if (
    instanceData.length >= state.instanceData.length &&
    hasPrefix(instanceData, state.instanceData)
  ) {
    if (instanceData.length > state.capacity) {
      state.capacity = Math.max(instanceData.length, state.capacity * 2, 3);
      gl.bufferData(
        gl.ARRAY_BUFFER,
        new Float32Array(state.capacity),
        gl.DYNAMIC_DRAW,
      );
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, toFloat32Array(instanceData));
    } else if (instanceData.length > state.instanceData.length) {
      gl.bufferSubData(
        gl.ARRAY_BUFFER,
        state.instanceData.length * 4,
        toFloat32Array(instanceData.subarray(state.instanceData.length)),
      );
    }
  } else {
    if (instanceData.length > state.capacity) {
      state.capacity = instanceData.length;
      gl.bufferData(
        gl.ARRAY_BUFFER,
        new Float32Array(state.capacity),
        gl.DYNAMIC_DRAW,
      );
    }
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, toFloat32Array(instanceData));
  }
  state.instanceCount = instanceData.length / 5;
  state.instanceData = instanceData;
  state.geometry = geometry;
  state.valueMin = valueMin;
  state.valueMax = valueMax;
}

export function drawHeatmap(
  gl: WebGL2RenderingContext,
  state: HeatmapRenderState,
  matrix: Float32Array,
): void {
  const { program, vao, instanceCount, uMatrix, uColorRamp } = state;
  gl.useProgram(program);

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

function buildHeatmapInstances(geometry: HeatmapGeometry): {
  valueMin: number;
  valueMax: number;
  instanceData: Float64Array;
} {
  let min = Infinity;
  let max = -Infinity;
  for (const item of geometry.series) {
    for (const cell of item.cells) {
      const cellValue = cell.z;
      if (cellValue < min) min = cellValue;
      if (cellValue > max) max = cellValue;
    }
  }
  if (!isFinite(min)) min = 0;
  if (!isFinite(max)) max = 1;
  const range = max !== min ? max - min : 1;

  const instances: number[] = [];
  for (const item of geometry.series) {
    for (const cell of item.cells) {
      instances.push(
        cell.x0,
        cell.y0,
        cell.x1,
        cell.y1,
        (cell.z - min) / range,
      );
    }
  }

  return {
    valueMin: min,
    valueMax: max,
    instanceData: Float64Array.from(instances),
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
