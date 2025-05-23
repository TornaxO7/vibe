@group(0) @binding(5)
var<uniform> color: vec4<f32>;

@fragment
fn main() -> @location(0) vec4<f32> {
    return color;
}