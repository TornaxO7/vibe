// Holds the screen resolution.
//   - `iResolution[0]`: Width
//   - `iResolution[1]`: Height
@group(0) @binding(0)
var<uniform> iResolution: vec2<f32>;

// Contains the presence of the playing audio.
// You can imagine this to be the height-value for the bar-shader.
//
// Note: You can get the length of the array with the `arrayLength` function: https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl-function-reference.html#func-arrayLength
@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

// Contains the time how long the shader has been running.
@group(1) @binding(1)
var<uniform> iTime: f32;

