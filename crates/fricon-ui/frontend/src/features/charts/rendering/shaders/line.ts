/** GLSL 300 es shaders for line chart rendering (GL_LINE_STRIP). */

export const lineVertexSource = `#version 300 es
precision highp float;

in vec2 a_position; // data-space (x, y)
uniform mat3 u_matrix;   // data → clip

void main() {
  vec3 pos = u_matrix * vec3(a_position, 1.0);
  gl_Position = vec4(pos.xy, 0.0, 1.0);
}
`;

export const lineFragmentSource = `#version 300 es
precision highp float;

uniform vec4 u_color;
out vec4 fragColor;

void main() {
  fragColor = u_color;
}
`;
