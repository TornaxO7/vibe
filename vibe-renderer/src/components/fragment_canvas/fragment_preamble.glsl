layout(set = 0, binding = 0) uniform vec2 iResolution;

layout(set = 1, binding = 0) uniform float iTime;

layout(set = 1, binding = 1) readonly buffer iAudio {
    float[] freqs;
};

layout(location = 0) out vec4 fragColor;
