// Holds the screen resolution.
//   - `iResolution[0]`: Width
//   - `iResolution[1]`: Height
@group(0) @binding(4)
var<uniform> iResolution: vec2<f32>;

// Contains the time how long the shader has been running.
@group(1) @binding(1)
var<uniform> iTime: f32;
