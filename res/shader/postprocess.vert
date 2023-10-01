#version 460 core

layout (location = 0) in vec2 i_pos;

layout (location = 0) out vec2 v_uv;

void main() {
  gl_Position = vec4(i_pos, 0., 1.);
  v_uv = (i_pos + 1.) * 0.5;
}