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

const float GAMMA = 2.2;
const vec3 WHITE = vec3(1.);

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;

    vec2 zuv = uv * float(freqs.length());
    int id = int(floor(zuv.x));
    float y = 1. - uv.y;

    // check if we are within a bar
    if (y <= freqs[id]) {
        vec3 bottom_color = sin(vec3(2., 4., 8.) + iTime) * .2 + .5;
        float presence = step(y, freqs[id]);

        vec3 col = mix(bottom_color, WHITE, y) * presence;

        // apply gamma correction
        col.x = pow(col.x, GAMMA);
        col.y = pow(col.y, GAMMA);
        col.z = pow(col.z, GAMMA);

        fragColor = vec4(col, y);
    }
}
