#version 460 core

layout (location = 0) in vec2 v_uv;

layout (location = 0) out vec4 f_color;

uniform sampler2D f_pos;
uniform sampler2D f_norm;
uniform sampler2D f_tint;

const int max_lights = 32;

uniform vec3 u_light_positions[max_lights];
uniform vec3 u_light_colors[max_lights];
uniform int u_n_lights;

uniform vec3 u_eye;

void main() {
  vec3 f_pos = texture(f_pos, v_uv).rgb;
  vec3 f_norm = texture(f_norm, v_uv).rgb;
  vec4 f_tint_full = texture(f_tint, v_uv);
  vec3 f_tint = f_tint_full.rgb;
  float f_spec = 1. / f_tint_full.a;

  const float ambient_strength = 0.5;
  vec3 ambient = ambient_strength * f_tint;

  vec3 diffuse = vec3(0.);
  vec3 specular = vec3(0.);
  for (int i = 0; i < u_n_lights; i++) {
    vec3 light_pos = u_light_positions[i];
    vec3 light_col = u_light_colors[i];

    vec3 light_dir = normalize(light_pos - f_pos);
    float diffuse_strength = max(dot(f_norm, light_dir), 0.) * 0.4;
    diffuse += diffuse_strength * light_col;

    vec3 view_dir = normalize(u_eye - f_pos);
    vec3 halfway_dir = normalize(light_dir + view_dir);
    float specular_strength = pow(max(dot(f_norm, halfway_dir), 0.), f_spec);
    specular += specular_strength * light_col;
  }

  vec3 res = ambient + diffuse + specular;
  f_color = vec4(res * f_tint, 1.);
}