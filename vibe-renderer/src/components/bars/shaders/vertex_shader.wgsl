@group(0) @binding(0)
var<uniform> bottom_left_corner: vec2f;

// Normalized
@group(0) @binding(1)
var<uniform> up_direction: vec2f;

// Not normalized.
// Length equals a full column "slot" (with the direction to the next column)
@group(0) @binding(2)
var<uniform> column_direction: vec2f;

// Length: The step required for the padding
@group(0) @binding(3)
var<uniform> padding: vec2f;

@group(0) @binding(4)
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
fn bass_treble(in: Input) -> Output {
    let freq_idx = in.instance_idx % arrayLength(&freqs);
    let freq = freqs[freq_idx];
    return inner(freq, in.vertex_idx, in.instance_idx);
}

@vertex
fn treble_bass(in: Input) -> Output {
    let freq_idx = in.instance_idx % arrayLength(&freqs);
    let freq = freqs[arrayLength(&freqs) - 1 - freq_idx];
    return inner(freq, in.vertex_idx, in.instance_idx);
}

fn inner(freq: f32, vertex_idx: u32, instance_idx: u32) -> Output {
    var pos: vec2f = bottom_left_corner;

    // x
    let is_bar_left_side = vertex_idx == 0 || vertex_idx == 2;
    if (is_bar_left_side) {
        pos += f32(instance_idx) * column_direction + padding;
    } else {
        pos += f32(instance_idx + 1) * column_direction - padding;
    }

    // y
    let is_top_vertex = vertex_idx <= 1; 
    if (is_top_vertex) {
        pos += up_direction * freq * max_height;
    }

    var output: Output;
    output.pos = vec4(pos, 0., 1.);
    output.bar_height = freq;

    return output;
}