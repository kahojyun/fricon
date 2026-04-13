/**
 * GLSL 300 es shaders for heatmap rendering via instanced quads.
 *
 * Each instance is a single heatmap cell. Per-instance attributes provide
 * the numeric cell bounds and the normalized value (0..1) which is mapped
 * to a color ramp in the fragment shader.
 */

export const heatmapVertexSource = `#version 300 es
precision highp float;

// Per-vertex: unit quad corners (0,0), (1,0), (1,1), (0,1)
in vec2 a_corner;

// Per-instance
in vec4 a_rect; // (x0, y0, x1, y1)
in float a_value;

uniform mat3 u_matrix; // maps data coords → clip space

out float v_value;

void main() {
  vec2 pos = vec2(
    mix(a_rect.x, a_rect.z, a_corner.x),
    mix(a_rect.y, a_rect.w, a_corner.y)
  );
  vec3 clip = u_matrix * vec3(pos, 1.0);
  gl_Position = vec4(clip.xy, 0.0, 1.0);
  v_value = a_value;
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
