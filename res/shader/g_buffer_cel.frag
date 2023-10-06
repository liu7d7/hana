#version 460 core

layout (location = 1) in vec3 v_norm;
layout (location = 2) in vec3 v_pos;

layout (location = 0) out vec4 f_pos;
layout (location = 1) out vec4 f_norm;
layout (location = 2) out ivec2 f_tint;

uniform int tint;

void main() {
  f_pos = vec4(v_pos, 1.);
  f_norm = vec4(normalize(v_norm), 1.);
  f_tint = ivec2(tint, 64);
}