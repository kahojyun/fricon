/** GLSL 300 es shaders for scatter chart rendering (GL_POINTS). */

export const scatterVertexSource = `#version 300 es
precision highp float;

in vec2 a_position; // data-space (x, y)
uniform mat3 u_matrix;   // data → clip
uniform float u_pointSize;

void main() {
  vec3 pos = u_matrix * vec3(a_position, 1.0);
  gl_Position = vec4(pos.xy, 0.0, 1.0);
  gl_PointSize = u_pointSize;
}
`;

export const scatterFragmentSource = `#version 300 es
precision highp float;

uniform vec4 u_color;
out vec4 fragColor;

void main() {
  // Circle mask: discard fragments outside the point radius
  vec2 cxy = 2.0 * gl_PointCoord - 1.0;
  if (dot(cxy, cxy) > 1.0) discard;
  fragColor = u_color;
}
`;
