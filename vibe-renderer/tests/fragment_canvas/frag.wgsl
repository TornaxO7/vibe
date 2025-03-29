@group(0) @binding(0)
var<uniform> iTime: f32;

@group(0) @binding(1)
var<uniform> iResolution: vec2<f32>;

@group(0) @binding(2)
var<storage, read> freqs: array<f32>;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let col = pos.xy / iResolution.xy + iTime + freqs[3];
    return vec4<f32>(col, 1., 1.);
}
