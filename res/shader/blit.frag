#version 460 core

layout (location = 0) in vec2 v_uv;

layout (location = 0) out vec4 f_color;

uniform sampler2D u_tex;

void main() {
  f_color = texture(u_tex, v_uv);
}