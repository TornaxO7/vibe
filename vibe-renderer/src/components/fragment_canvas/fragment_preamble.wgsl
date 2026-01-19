// Holds the screen resolution.
//   - `iResolution[0]`: Width
//   - `iResolution[1]`: Height
@group(0) @binding(0)
var<uniform> iResolution: vec2f;

// Contains the presence of the playing audio.
// You can imagine this to be the height-value for the bar-shader.
//
// Note: You can get the length of the array with the `arrayLength` function: https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl-function-reference.html#func-arrayLength
@group(0) @binding(1)
var<storage, read> freqs: array<f32>;

// Contains the time how long the shader has been running.
@group(0) @binding(2)
var<uniform> iTime: f32;

// Contains the (x, y) coordinate of the mouse.
// `x` and `y` are within the range [0, 1]:
//   - (0, 0) => top left corner
//   - (1, 0) => top right corner
//   - (0, 1) => bottom left corner
//   - (1, 1) => bottom right corner
@group(0) @binding(3)
var<uniform> iMouse: vec2f;

@group(0) @binding(4)
var iSampler: sampler;

@group(0) @binding(5)
var iTexture: texture_2d<f32>;
