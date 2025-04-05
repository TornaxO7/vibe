@group(0) @binding(0)
var<uniform> column_width: f32;

@group(0) @binding(1)
var<uniform> padding: f32;

@group(0) @binding(2)
var<uniform> max_height: f32;

@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

struct Input {
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
};

struct Output {
    @builtin(position) pos: vec4<f32>,
    @location(0) bar_height: f32,
};

// Assuming:
//
// 0    1
//  ----
//  |\ |
//  | \|
//  ----
// 2    3
@vertex
fn main(in: Input) -> Output {
    var pos = vec2<f32>(-1., -1.);

    // x
    if (in.vertex_idx == 0 || in.vertex_idx == 2) {
        pos.x += f32(in.instance_idx) * column_width + padding;
    } else {
        pos.x += f32(in.instance_idx + 1) * column_width - padding;
    }

    // y
    if (in.vertex_idx <= 1) {
        const CANVAS_HEIGHT: f32 = 2.;
        pos.y += CANVAS_HEIGHT * freqs[in.instance_idx] * max_height;
    }

    var output: Output;
    output.pos = vec4(pos, 0., 1.);
    output.bar_height = freqs[in.instance_idx];

    return output;
}
