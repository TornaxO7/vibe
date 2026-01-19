// Holds the screen resolution.
//   - `iResolution[0]`: Width
//   - `iResolution[1]`: Height
layout(set = 0, binding = 0) uniform vec2 iResolution;

// Contains the presence of the playing audio.
// You can imagine this to be the height-value for the bar-shader.
//
// Note: You can get the length of the array `freqs.length()`
layout(set = 0, binding = 1) readonly buffer iAudio {
    float[] freqs;
};

// Contains the time how long the shader has been running.
layout(set = 0, binding = 2) uniform float iTime;

// Contains the (x, y) coordinate of the mouse.
// `x` and `y` are within the range [0, 1]:
//   - (0, 0) => top left corner
//   - (1, 0) => top right corner
//   - (0, 1) => bottom left corner
//   - (1, 1) => bottom right corner
layout(set = 0, binding = 3) uniform vec2 iMouse;

layout(set = 0, binding = 4) uniform sampler iSampler;

layout(set = 0, binding = 5) uniform texture2D iTexture;

// The color for the fragment/pixel.
// Needs to be set in your shader (like in shadertoy).
layout(location = 0) out vec4 fragColor;
