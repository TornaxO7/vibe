@group(0) @binding(5)
var<uniform> top_color: vec4<f32>;

@group(0) @binding(6)
var<uniform> bottom_color: vec4<f32>;

struct Input {
    @builtin(position) pos: vec4<f32>,
    @location(0) bar_height: f32,
};

@fragment
fn main(in: Input) -> @location(0) vec4<f32> {
    return mix(top_color, bottom_color, 1. - in.bar_height);
}