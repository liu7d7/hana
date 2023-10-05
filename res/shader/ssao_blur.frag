#version 460 core

layout (location = 0) in vec2 v_uv;

out float f_color;

uniform sampler2D u_ssao_input;
uniform vec2 u_input_size;

void main() {
  vec2 texelSize = 1.0 / vec2(u_input_size);
  float result = 0.0;
  for (int x = -3; x <= 3; ++x) {
    for (int y = -3; y <= 3; ++y) {
      vec2 offset = vec2(float(x), float(y)) * texelSize;
      result += texture(u_ssao_input, v_uv + offset).r;
    }
  }
  f_color = result / (7.0 * 7.0);
}