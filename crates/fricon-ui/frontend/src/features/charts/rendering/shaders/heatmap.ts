/**
 * GLSL 300 es shaders for heatmap rendering via instanced quads.
 *
 * Each instance is a single heatmap cell. Per-instance attributes provide
 * the cell position (col, row) and the normalized value (0..1) which is
 * mapped to a color ramp in the fragment shader.
 */

export const heatmapVertexSource = `#version 300 es
precision highp float;

// Per-vertex: unit quad corners (0,0), (1,0), (1,1), (0,1)
in vec2 a_corner;

// Per-instance
in vec3 a_cell; // (col, row, normalizedValue)

uniform mat3 u_matrix; // maps (col, row) grid coords → clip space
uniform vec2 u_cellSize; // (1.0 / numCols, 1.0 / numRows) in grid-coord units — unused, we just offset by 1

out float v_value;

void main() {
  // Cell spans from (col, row) to (col+1, row+1) in grid coordinates
  vec2 pos = a_cell.xy + a_corner;
  vec3 clip = u_matrix * vec3(pos, 1.0);
  gl_Position = vec4(clip.xy, 0.0, 1.0);
  v_value = a_cell.z;
}
`;

export const heatmapFragmentSource = `#version 300 es
precision highp float;

in float v_value;
out vec4 fragColor;

// Color ramp: 5 stops matching ECharts defaults
// #2c7bb6, #abd9e9, #ffffbf, #fdae61, #d7191c
uniform vec3 u_colorRamp[5];

void main() {
  float t = clamp(v_value, 0.0, 1.0);
  float scaled = t * 4.0; // 0..4 for 5 stops
  int idx = int(floor(scaled));
  float frac = scaled - float(idx);

  vec3 c;
  if (idx >= 4) {
    c = u_colorRamp[4];
  } else {
    c = mix(u_colorRamp[idx], u_colorRamp[idx + 1], frac);
  }
  fragColor = vec4(c, 1.0);
}
`;
