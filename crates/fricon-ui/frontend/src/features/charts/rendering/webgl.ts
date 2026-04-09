/** Low-level WebGL2 utilities: shader compilation, buffer management, uniforms, resize. */

export function createShader(
  gl: WebGL2RenderingContext,
  type: number,
  source: string,
): WebGLShader {
  const shader = gl.createShader(type);
  if (!shader) throw new Error("Failed to create shader");
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const info = gl.getShaderInfoLog(shader);
    gl.deleteShader(shader);
    throw new Error(`Shader compile error: ${info}`);
  }
  return shader;
}

export function createProgram(
  gl: WebGL2RenderingContext,
  vertexSource: string,
  fragmentSource: string,
): WebGLProgram {
  const vs = createShader(gl, gl.VERTEX_SHADER, vertexSource);
  const fs = createShader(gl, gl.FRAGMENT_SHADER, fragmentSource);
  const program = gl.createProgram();
  if (!program) throw new Error("Failed to create program");
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    const info = gl.getProgramInfoLog(program);
    gl.deleteProgram(program);
    throw new Error(`Program link error: ${info}`);
  }
  // Shaders can be detached after linking
  gl.detachShader(program, vs);
  gl.detachShader(program, fs);
  gl.deleteShader(vs);
  gl.deleteShader(fs);
  return program;
}

export function createBuffer(
  gl: WebGL2RenderingContext,
  data: Float32Array,
  usage: number = gl.STATIC_DRAW,
): WebGLBuffer {
  const buffer = gl.createBuffer();
  if (!buffer) throw new Error("Failed to create buffer");
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, data, usage);
  return buffer;
}

export function resizeCanvas(canvas: HTMLCanvasElement): boolean {
  const dpr = window.devicePixelRatio || 1;
  const displayWidth = Math.round(canvas.clientWidth * dpr);
  const displayHeight = Math.round(canvas.clientHeight * dpr);
  if (canvas.width !== displayWidth || canvas.height !== displayHeight) {
    canvas.width = displayWidth;
    canvas.height = displayHeight;
    return true;
  }
  return false;
}

/** Chart area margins in CSS pixels (axes labels, padding). */
export interface ChartMargin {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export const DEFAULT_MARGIN: ChartMargin = {
  top: 20,
  right: 20,
  bottom: 40,
  left: 60,
};

export const HEATMAP_MARGIN: ChartMargin = {
  top: 20,
  right: 80,
  bottom: 40,
  left: 60,
};

/** Convert CSS-pixel chart area to GL clip-space viewport coords. */
export function chartAreaToViewport(
  canvas: HTMLCanvasElement,
  margin: ChartMargin,
): { x: number; y: number; width: number; height: number } {
  const dpr = window.devicePixelRatio || 1;
  const x = Math.round(margin.left * dpr);
  const y = Math.round(margin.bottom * dpr);
  const width = Math.round(
    (canvas.clientWidth - margin.left - margin.right) * dpr,
  );
  const height = Math.round(
    (canvas.clientHeight - margin.top - margin.bottom) * dpr,
  );
  return { x, y, width, height };
}

/**
 * Build a 3×3 transformation matrix (column-major Float32Array) that maps
 * data coordinates [xMin..xMax, yMin..yMax] to GL clip space [-1..1, -1..1].
 */
export function dataToClipMatrix(
  xMin: number,
  xMax: number,
  yMin: number,
  yMax: number,
): Float32Array {
  const sx = xMax !== xMin ? 2 / (xMax - xMin) : 1;
  const sy = yMax !== yMin ? 2 / (yMax - yMin) : 1;
  const tx = -(xMax + xMin) / (xMax - xMin) || 0;
  const ty = -(yMax + yMin) / (yMax - yMin) || 0;
  // Column-major 3×3
  // prettier-ignore
  return new Float32Array([
    sx, 0,  0,
    0,  sy, 0,
    tx, ty, 1,
  ]);
}

/** Multiply two column-major 3×3 matrices: result = a * b */
export function mul3x3(a: Float32Array, b: Float32Array): Float32Array {
  const out = new Float32Array(9);
  for (let col = 0; col < 3; col++) {
    for (let row = 0; row < 3; row++) {
      out[col * 3 + row] =
        a[0 * 3 + row] * b[col * 3 + 0] +
        a[1 * 3 + row] * b[col * 3 + 1] +
        a[2 * 3 + row] * b[col * 3 + 2];
    }
  }
  return out;
}

/** Build a 3×3 zoom/pan matrix from chart-local axis transforms in pixel space. */
export function zoomToClipMatrix(
  scaleX: number,
  translateX: number,
  scaleY: number,
  translateY: number,
  viewportWidth: number,
  viewportHeight: number,
): Float32Array {
  // Chart interaction is defined in chart-area pixel space:
  //   px' = scaleX * px + translateX
  //   py' = scaleY * py + translateY
  //
  // Our data matrix maps directly to clip space, so the zoom matrix must
  // reproduce that same pixel-space transform after clip conversion.
  //
  // For x: clip = 2 * px / width - 1
  //   => clip' = scaleX * clip + (scaleX - 1) + 2 * translateX / width
  //
  // For y we use top-origin pixel coordinates while clip space is bottom-origin:
  //   clip = 1 - 2 * py / height
  //   => clip' = scaleY * clip + (1 - scaleY) - 2 * translateY / height
  const sx = scaleX;
  const sy = scaleY;
  const clipTx = scaleX - 1 + (2 * translateX) / viewportWidth;
  const clipTy = 1 - scaleY - (2 * translateY) / viewportHeight;
  // prettier-ignore
  return new Float32Array([
    sx, 0,  0,
    0,  sy, 0,
    clipTx, clipTy, 1,
  ]);
}

/** Parse a CSS hex color (#rrggbb or #rgb) into [r, g, b] in 0..1. */
export function hexToRgb(hex: string): [number, number, number] {
  let r = 0,
    g = 0,
    b = 0;
  if (hex.length === 4) {
    r = parseInt(hex[1] + hex[1], 16) / 255;
    g = parseInt(hex[2] + hex[2], 16) / 255;
    b = parseInt(hex[3] + hex[3], 16) / 255;
  } else if (hex.length === 7) {
    r = parseInt(hex.slice(1, 3), 16) / 255;
    g = parseInt(hex.slice(3, 5), 16) / 255;
    b = parseInt(hex.slice(5, 7), 16) / 255;
  }
  return [r, g, b];
}

/** Default series color palette. */
export const SERIES_COLORS = [
  "#5470c6",
  "#91cc75",
  "#fac858",
  "#ee6666",
  "#73c0de",
  "#3ba272",
  "#fc8452",
  "#9a60b4",
  "#ea7ccc",
];

export const LIVE_NEWEST_COLOR = "#2563eb";
export const LIVE_OLD_COLOR = "#94a3b8";
