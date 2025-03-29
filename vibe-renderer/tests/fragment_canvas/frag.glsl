layout(set = 0, binding = 0) uniform float iTime;

layout(set = 0, binding = 1) uniform vec2 iResolution;

layout(set = 0, binding = 2) readonly buffer iAudio {
    float[] freqs;
};

layout(location = 0) out vec4 fragColor;

void main() {
    vec2 col = gl_FragCoord.xy / iResolution.xy * iTime + freqs[3];
    fragColor = vec4(col, 1., 1.);
}
