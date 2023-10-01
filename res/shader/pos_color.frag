#version 460 core

layout (location = 0) in vec4 v_tint;
layout (location = 1) in vec3 v_norm;
layout (location = 2) in vec3 v_pos;

uniform vec3 u_light_pos;
uniform vec3 u_light_color;
uniform vec3 u_eye;

out vec4 f_color;

void main() {
  float ambient_strength = 0.5;
  vec3 ambient = ambient_strength * u_light_color;

  vec3 norm = normalize(v_norm);
  vec3 light_dir = normalize(u_light_pos - v_pos);
  float diffuse_strength = max(dot(norm, light_dir), 0.0) * 0.4;
  vec3 diffuse = diffuse_strength * u_light_color;

  vec3 view_dir = normalize(u_eye - v_pos);
  vec3 halfway_dir = normalize(light_dir + view_dir);

  float specular_strength = pow(max(dot(v_norm, halfway_dir), 0.), 128);
  vec3 specular = specular_strength * u_light_color;

  vec3 res = ambient + diffuse + specular;
  f_color = vec4(res, 1.) * v_tint;
}