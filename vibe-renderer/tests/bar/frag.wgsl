@group(1) @binding(1)
var<uniform> iTime: f32;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(iTime, 0., 1., 1.);
}
