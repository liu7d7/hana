#version 460 core

layout (location = 0) in vec4 v_tint;
layout (location = 1) in vec3 v_norm;
layout (location = 2) in vec3 v_pos;

layout (location = 0) out vec4 f_pos;
layout (location = 1) out vec4 f_norm;
layout (location = 2) out vec4 f_tint;

void main() {
  f_pos = vec4(v_pos, 1.);
  f_norm = vec4(normalize(v_norm), 1.);
  f_tint = vec4(v_tint.rgb, 1./64.);
}