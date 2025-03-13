@group(0) @binding(0)
var<uniform> iTime: f32;

// x: width
// y: height
@group(1) @binding(0)
var<uniform> iResolution: vec2<f32>;

@group(2) @binding(0)
var<storage, read> iAudio: array<f32>;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(0.);
}