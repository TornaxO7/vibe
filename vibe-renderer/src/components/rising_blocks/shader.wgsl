struct VertexParams {
    // Not normalized.
    // Length equals a full column "slot" (with the direction to the next column)
    column_direction: vec2f,
    bottom_left_corner: vec2f,

    // length = min height size
    up_direction: vec2f,

    time: f32,
    amount_columns: u32,
};

struct FragmentParams {
};

struct BlockData {
    // The time when the block was created
    @location(0) start_time: f32,
    // The current height of the block
    @location(1) height: f32,
};

@group(0) @binding(0)
var<uniform> vp: VertexParams;

// @group(0) @binding(1)
// var<uniform> fp: FragmentParams;

struct Input {
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,

    data: BlockData,
};

struct Output {
    @builtin(position) pos: vec4<f32>,
    // The relative position within the spectrum.
    // `pos` but `x` and `y` are normalized (aka. they are in the range [0, 1])
    @location(0) rel_pos: vec2f,
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
    let is_left_channel = in.instance_idx < vp.amount_columns;
    let is_bar_left_side = in.vertex_idx == 0 || in.vertex_idx == 2;

    var pos = vp.bottom_left_corner;
    if (is_bar_left_side) {
        pos += f32(in.instance_idx) * vp.column_direction + padding;

        if (is_left_channel) {
            output.rel_pos.x = f32(in.instance_idx) / f32(vp.amount_columns);
        } else {
            output.rel_pos.x = f32(vp.amount_columns*2 - in.instance_idx) / f32(vp.amount_columns);
        }
    } else {
        pos += f32(in.instance_idx + 1) * vp.column_direction - padding;

        if (is_left_channel) {
            output.rel_pos.x = f32(in.instance_idx + 1) / f32(vp.amount_columns);
        } else {
            output.rel_pos.x = f32(vp.amount_columns*2 + 1 - in.instance_idx) / f32(vp.amount_columns);
        }
    }

    // -- y
    let is_bottom_vertex = in.vertex_idx >= 2;
    if (is_bottom_vertex) {
        pos -= vp.up_direction * in.data.height;
    }

    // steadily go up
    pos += vp.up_direction * (vp.time - in.data.start_time);

    output.pos = vec4f(pos, 0., 1.);
    return output;
}

// // == fragment ==
// // Idea: Create a function which returns a value from [0., 1.]:
// // - 0. => Use bar color
// // - 1. => Use border color
// //
// // Basically something like a SDF for the bar
// //
// // Assumptions:
// // - Bar x coord is within [-1, 1]
// // - Height is within range [0, freq]
// fn get_border_mask(pos: vec2f, freq: f32) -> f32 {
//     // let border_width = .1;
//     const BORDER_TRANSITION_SIZE: f32 = .01;

//     // horizontal mask
//     let border_width_transition_start = clamp(1. - fp.border_width, 0., 1.);
//     let border_width_transition_end = border_width_transition_start + BORDER_TRANSITION_SIZE;
//     let width = smoothstep(border_width_transition_start, border_width_transition_end, abs(pos.x));

//     // vertical mask
//     const HEIGHT_FACTOR: f32 = .01; // found out by experimenting... seems to be fine, lol
//     let border_height_transition_start = max(freq - fp.border_width*HEIGHT_FACTOR - BORDER_TRANSITION_SIZE, 0.);
//     let border_height_transition_end = max(freq - fp.border_width*HEIGHT_FACTOR, 0.);
//     let height = smoothstep(border_height_transition_start, border_height_transition_end, pos.y);

//     return max(width, height);
// }

@fragment
fn fs_main(in: Output) -> @location(0) vec4f {
    return vec4f(1.);
}

