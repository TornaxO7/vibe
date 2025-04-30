// Holds the screen resolution.
//   - `iResolution[0]`: Width
//   - `iResolution[1]`: Height
layout(set = 0, binding = 0) uniform vec2 iResolution;

// Contains the presence of the playing audio.
// You can imagine this to be the height-value for the bar-shader.
//
// Note: You can get the length of the array `freqs.length()`
layout(set = 1, binding = 0) readonly buffer iAudio {
    float[] freqs;
};

// Contains the time how long the shader has been running.
layout(set = 1, binding = 1) uniform float iTime;

// The color for the fragment/pixel.
// Needs to be set in your shader (like in shadertoy).
layout(location = 0) out vec4 fragColor;
