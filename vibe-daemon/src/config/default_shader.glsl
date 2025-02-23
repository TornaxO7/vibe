// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.
layout(binding = 0) readonly buffer iAudio {
    float[] freqs;
};

// x: width
// y: height
layout(binding = 1) uniform vec2 iResolution;

layout(binding = 2) uniform float iTime;

// the color which the pixel should have
layout(location = 0) out vec4 fragColor;

void main() {
    float gamma = 2.2;
    float m = 0.;

    vec2 uv = gl_FragCoord.xy / iResolution.xy;

    vec2 zuv = uv * float(freqs.length());
    int id = int(floor(zuv.x));
    float y = 1. - uv.y;

    m += step(y, freqs[id]);

    vec3 bottom_color = sin(vec3(12., 23., 34.) + iTime) * .2 + .5;
    vec3 top_color = vec3(1.);

    if (m > 0.) {
        vec3 col = mix(bottom_color, top_color, y) * m;
        // apply gamma correction
        col.x = pow(col.x, gamma);
        col.y = pow(col.y, gamma);
        col.z = pow(col.z, gamma);
        fragColor = vec4(col, y);
    }
}
