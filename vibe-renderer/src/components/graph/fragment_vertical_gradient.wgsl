@group(0) @binding(0)
var<uniform> canvas_height: f32;

@group(0) @binding(1)
var<uniform> max_height: f32;

@group(0) @binding(6)
var<uniform> color_top: vec4f;

@group(0) @binding(7)
var<uniform> color_bottom: vec4f;

@group(0) @binding(8)
var<uniform> smoothness: f32;

@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

@fragment
fn main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let bar_height = freqs[u32(pos.x)];

    let relative_y = 1. - pos.y / canvas_height;
    let presence = smoothstep(relative_y - smoothness, relative_y, bar_height * max_height);
    let relative_graph_height = relative_y / (bar_height * max_height);

    return mix(color_bottom, color_top, relative_graph_height) * presence;
}
