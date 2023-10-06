#version 460 core

layout (location = 0) in vec2 v_uv;

layout (location = 0) out vec4 f_color;

const int max_steps = 12;
uniform float steps[max_steps];
uniform int n_steps;
uniform sampler2D u_tex;

vec3 to_rgb(vec3 c) {
	vec4 K = vec4(1., 2. / 3., 1. / 3., 3.);
	return c.z * mix(K.xxx, clamp(abs(fract(c.x + K.xyz) * 6. - K.w) - K.x, 0., 1.), c.y);
}

vec3 to_hsv(vec3 c) {
	float cMax = max(max(c.r, c.g),c.b),
	      cMin = min(min(c.r, c.g),c.b),
	      delta = cMax - cMin;
	vec3 hsv = vec3(0., 0., cMax);
	if (cMax > cMin) {
		hsv.y = delta / cMax;
		if (c.r == cMax) {
			hsv.x = (c.g - c.b) / delta;
		} else if (c.g == cMax) {
			hsv.x = 2. + (c.b - c.r) / delta;
		} else {
			hsv.x = 4. + (c.r - c.g) / delta;
		}
		hsv.x = fract(hsv.x / 6.);
	}

	return hsv;
}

void main() {
  vec3 hsv = to_hsv(texture(u_tex, v_uv).rgb);

  for (int i = 1; i < n_steps; i++) {
    if (hsv.z > steps[i - 1] && hsv.z <= steps[i]) {
      hsv.z = steps[i];
    }
  }

  f_color = vec4(to_rgb(hsv), 1.);
}