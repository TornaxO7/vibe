struct VertexParams {
    bottom_left_corner: vec2f,
    // Normalized
    up_direction: vec2f,
    // Not normalized.
    // Length equals a full column "slot" (with the direction to the next column)
    column_direction: vec2f,
    // Length: The step required for the padding
    padding: vec2f,
    max_height: f32,
    height_mirrored: u32,
};

@group(0) @binding(0)
var<uniform> vp: VertexParams;

@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

const TRUE: u32 = 1;

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
    var pos: vec2f = vp.bottom_left_corner;

    // x
    let is_bar_left_side = vertex_idx == 0 || vertex_idx == 2;
    if (is_bar_left_side) {
        pos += f32(instance_idx) * vp.column_direction + vp.padding;
    } else {
        pos += f32(instance_idx + 1) * vp.column_direction - vp.padding;
    }

    // y
    let is_top_vertex = vertex_idx <= 1; 
    if (is_top_vertex) {
        pos += vp.up_direction * freq * vp.max_height;
    } else if (vp.height_mirrored == TRUE) {
        pos += -vp.up_direction * freq * vp.max_height;
    }

    var output: Output;
    output.pos = vec4(pos, 0., 1.);
    output.bar_height = freq;

    return output;
}