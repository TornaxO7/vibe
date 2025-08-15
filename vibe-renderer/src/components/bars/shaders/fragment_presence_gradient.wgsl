@group(0) @binding(7)
var<uniform> top_color: vec4<f32>;

@group(0) @binding(8)
var<uniform> bottom_color: vec4<f32>;

struct Input {
    @builtin(position) pos: vec4<f32>,
    @location(0) bar_height: f32,
};

@fragment
fn main(in: Input) -> @location(0) vec4<f32> {
    return mix(top_color, bottom_color, clamp(0., 1., 1. - in.bar_height));
}