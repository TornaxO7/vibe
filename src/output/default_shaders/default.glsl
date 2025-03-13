layout(set = 0, binding = 0) uniform float iTime;

// x: width
// y: height
layout(set = 1, binding = 0) uniform vec2 iResolution;

// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.
layout(set = 2, binding = 0) readonly buffer iAudio {
    float[] freqs;
};

// the color which the pixel should have
layout(location = 0) out vec4 fragColor;

void main() {
    const float GAMMA = 2.2;
    const vec3 WHITE = vec3(1.);

    vec2 uv = gl_FragCoord.xy / iResolution.xy;

    vec2 zuv = uv * float(freqs.length());
    float x = fract(zuv.x);
    float y = 1. - uv.y;

    int id = int(floor(zuv.x));
    float freq = freqs[id] * .75;

    // left and right space of a bar
    float space = 0.2;
    
    // check if we are within a bar
    if (y <= freq && space < x && x < 1. - space) {
        vec3 bottom_color = sin(vec3(2., 4., 8.) * iTime * .25) * .2 + .6;
        float presence = step(y, freq);
    
        vec3 col = mix(bottom_color, WHITE, y) * presence;
    
        // apply gamma correction
        col.x = pow(col.x, GAMMA);
        col.y = pow(col.y, GAMMA);
        col.z = pow(col.z, GAMMA);
        fragColor = vec4(col, y);
    }
}
