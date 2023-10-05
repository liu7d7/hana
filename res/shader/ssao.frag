

#version 460 core

layout (location = 0) in vec2 v_uv;

out float f_color;

uniform sampler2D f_pos;
uniform sampler2D f_norm;
uniform sampler2D u_noise;

uniform vec3 u_samples[64];

// parameters (you'd probably want to use them as uniforms to more easily tweak the effect)
int kernelSize = 64;
float radius = 4;
float bias = 0.01;

// tile noise texture over screen based on screen dimensions divided by noise size
const vec2 noiseScale = vec2(2304.0/8.0, 1440.0/8.0);

uniform mat4 projection;

void main() {
  // get input for SSAO algorithm
  vec3 fragPos = texture(f_pos, v_uv).xyz;
  vec3 normal = normalize(texture(f_norm, v_uv).rgb);
  vec3 randomVec = normalize(texture(u_noise, v_uv * noiseScale).xyz);
  // create TBN change-of-basis matrix: from tangent-space to view-space
  vec3 tangent = normalize(randomVec - normal * dot(randomVec, normal));
  vec3 bitangent = cross(normal, tangent);
  mat3 TBN = mat3(tangent, bitangent, normal);
  // iterate over the sample kernel and calculate occlusion factor
  float occlusion = 0.0;
  for (int i = 0; i < kernelSize; ++i) {
    // get sample position
    vec3 samplePos = TBN * u_samples[i]; // from tangent to view-space
    samplePos = fragPos + samplePos * radius;

    // project sample position (to sample texture) (to get position on screen/texture)
    vec4 offset = vec4(samplePos, 1.0);
    offset = projection * offset; // from view to clip-space
    offset.xyz /= offset.w; // perspective divide
    offset.xyz = offset.xyz * 0.5 + 0.5; // transform to range 0.0 - 1.0

    // get sample depth
    float sampleDepth = texture(f_pos, offset.xy).z; // get depth value of kernel sample

    // range check & accumulate
    float rangeCheck = smoothstep(0.0, 1.0, radius / abs(fragPos.z - sampleDepth));
    occlusion += (sampleDepth >= samplePos.z + bias ? 1.0 : 0.0) * rangeCheck;
  }
  occlusion = 1.0 - (occlusion / kernelSize);

  f_color = occlusion;
}

