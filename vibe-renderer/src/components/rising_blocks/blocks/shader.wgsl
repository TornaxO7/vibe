struct VertexParams {
    // Not normalized.
    // Length equals a full column "slot" (with the direction to the next column)
    column_direction: vec2f,
    bottom_left_corner: vec2f,

    // length = min height size
    up_direction: vec2f,
    block_height: vec2f,

    time: f32,
    amount_columns: u32,
};

@group(0) @binding(0)
var<uniform> vp: VertexParams;

struct Input {
    @builtin(vertex_index) vertex_idx: u32,

    // The time when the block was created
    @location(0) start_time: f32,
    @location(1) column_idx: u32,
};

struct Output {
    @builtin(position) pos: vec4<f32>,
};

// Assuming:
//
//   ^ y
//   |   (top)
//   |  0    1
// 0-|-- ---- -----> x
//   |   |\ |
//   |   | \|     ^
//   |   ----     | Goes up
//   |  2    3    |
//   | (bottom)
//
@vertex
fn vs_main(in: Input) -> Output {
    var output: Output;

    let padding = vp.column_direction * .2;

    // -- x
    let is_left_channel = in.column_idx < vp.amount_columns;
    let is_bar_left_side = in.vertex_idx == 0 || in.vertex_idx == 2;

    var pos = vp.bottom_left_corner;
    if (is_bar_left_side) {
        pos += f32(in.column_idx) * vp.column_direction + padding;
    } else {
        pos += f32(in.column_idx + 1) * vp.column_direction - padding;
    }

    // -- y
    let is_bottom_vertex = in.vertex_idx >= 2;
    if (is_bottom_vertex) {
        pos -= vp.block_height;
    }

    // steadily go up
    pos += vp.up_direction * (vp.time - in.start_time);

    output.pos = vec4f(pos, 0., 1.);
    return output;
}

// == fragment ==
@fragment
fn fs_main(in: Output) -> @location(0) vec4f {
    return vec4f(1.);
}

