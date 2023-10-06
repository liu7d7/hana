#version 460 core

layout (location = 0) in vec2 v_uv;

layout (location = 0) out vec4 f_color;

uniform sampler2D f_pos;
uniform sampler2D f_norm;
uniform isampler2D f_tint;

uniform vec3 u_eye;

const int n_colors = 64;
uniform vec3 palette[n_colors * 4];

const float thresholds[5] = { 0., 0.3, 0.475, 0.65, 1. };

void main() {
  ivec2 f_tint_full = texture(f_tint, v_uv).xy;
  if (f_tint_full.y == 0) {
    discard;
  }

  vec3 f_pos = texture(f_pos, v_uv).rgb;
  vec3 f_norm = texture(f_norm, v_uv).rgb;
  int f_tint = f_tint_full.x;
  float f_spec = float(f_tint_full.y);

  const float ambient_strength = 0.3;
  float ambient = ambient_strength;

  const vec3 light_dir = normalize(vec3(0.66, 2, 0.66));
  float diffuse = max(dot(f_norm, light_dir), 0.) * 0.4;

  vec3 view_dir = normalize(u_eye - f_pos);
  vec3 halfway_dir = normalize(light_dir + view_dir);
  float specular = pow(max(dot(f_norm, halfway_dir), 0.), f_spec);

  float res = clamp(ambient + diffuse + specular, 0., 1.);

  int shade = 0;
  for (int i = 1; i < 5; i++) {
    if (res > thresholds[i - 1] && res <= thresholds[i]) {
      shade = i - 1;
      break;
    }
  }

  f_color = vec4(palette[f_tint * 4 + 3 - shade], 1.);
}