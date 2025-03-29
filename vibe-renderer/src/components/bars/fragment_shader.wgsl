@group(1) @binding(0)
var<uniform> iTime: f32;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let bottom_color = sin(vec3<f32>(2., 4., 8.) * iTime * .25) * .2 + .6;
    return vec4<f32>(bottom_color, pos.y);
}
