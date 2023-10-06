#version 460 core

layout (location = 0) in vec3 i_pos;
layout (location = 1) in vec3 i_norm;

layout (location = 1) out vec3 v_norm;
layout (location = 2) out vec3 v_pos;

uniform mat4 u_proj;
uniform mat4 u_look;

void main() {
  gl_Position = u_proj * u_look * vec4(i_pos, 1.0);
  v_norm = i_norm;
  v_pos = i_pos;
}