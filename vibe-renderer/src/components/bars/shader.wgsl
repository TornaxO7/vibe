struct VertexParams {
    // Not normalized.
    // Length equals a full column "slot" (with the direction to the next column)
    column_direction: vec2f,
    bottom_left_corner: vec2f,
    // Normalized
    up_direction: vec2f,
    max_height: f32,
    height_mirrored: u32,
    amount_bars: u32,
};

struct FragmentParams {
    color1: vec4f,
    color2: vec4f,

    border_color: vec4f,
    border_width: f32,
};

@group(0) @binding(0)
var<uniform> vp: VertexParams;

@group(0) @binding(1)
var<uniform> fp: FragmentParams;

// In its own group, due to left (1st source) and right (2nd source) half of bars
@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

const TRUE: u32 = 1;

struct Input {
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
};

struct Output {
    @builtin(position) pos: vec4<f32>,
    @location(0) freq: f32,
    // The relative position within the spectrum.
    // `pos` but `x` and `y` are normalized (aka. they are in the range [0, 1])
    @location(1) rel_pos: vec2f,
    // An internal "coordinate-system" of the bar:
    // - (-1., 0)      => bottom left corner of the bar
    // - (-1., <freq>) => top left corner of the bar
    // - (1. , 0.)     => bottom right corner of the bar
    // - (1. , <freq>) => top right corner of the bar
    @location(2) internal_pos: vec2f,
};

// Assuming:
//
// ^ y
// |
// |  0    1
// |   ----
// |   |\ |
// |   | \|
// |   ----
// |  2    3
// ----------> x
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
    var output: Output;
    output.freq = freq;

    let padding = vp.column_direction * .2;

    // x
    let is_left_channel = instance_idx < vp.amount_bars;
    let is_bar_left_side = vertex_idx == 0 || vertex_idx == 2;

    var pos = vp.bottom_left_corner;
    if (is_bar_left_side) {
        pos += f32(instance_idx) * vp.column_direction + padding;
        output.internal_pos.x = -1.;

        if (is_left_channel) {
            output.rel_pos.x = f32(instance_idx) / f32(vp.amount_bars);
        } else {
            output.rel_pos.x = f32(vp.amount_bars*2 - instance_idx) / f32(vp.amount_bars);
        }
    } else {
        pos += f32(instance_idx + 1) * vp.column_direction - padding;
        output.internal_pos.x = 1.;

        if (is_left_channel) {
            output.rel_pos.x = f32(instance_idx + 1) / f32(vp.amount_bars);
        } else {
            output.rel_pos.x = f32(vp.amount_bars*2 + 1 - instance_idx) / f32(vp.amount_bars);
        }
    }

    // y
    let is_top_vertex = vertex_idx <= 1; 
    if (is_top_vertex) {
        pos += vp.up_direction * freq * vp.max_height;
        output.rel_pos.y = freq;
        output.internal_pos.y = freq;
    } else if (vp.height_mirrored == TRUE) {
        pos += -vp.up_direction * freq * vp.max_height;
    }

    output.pos = vec4f(pos, 0., 1.);

    return output;
}

// == fragment ==
// Idea: Create a function which returns a value from [0., 1.]:
// - 0. => Use bar color
// - 1. => Use border color
//
// Basically something like a SDF for the bar
//
// Assumptions:
// - Bar x coord is within [-1, 1]
// - Height is within range [0, freq]
fn get_border_mask(pos: vec2f, freq: f32) -> f32 {
    // let border_width = .1;
    const BORDER_TRANSITION_SIZE: f32 = .01;

    // horizontal mask
    let border_width_transition_start = clamp(1. - fp.border_width, 0., 1.);
    let border_width_transition_end = border_width_transition_start + BORDER_TRANSITION_SIZE;
    let width = smoothstep(border_width_transition_start, border_width_transition_end, abs(pos.x));

    // vertical mask
    const HEIGHT_FACTOR: f32 = .01; // found out by experimenting... seems to be fine, lol
    let border_height_transition_start = max(freq - fp.border_width*HEIGHT_FACTOR - BORDER_TRANSITION_SIZE, 0.);
    let border_height_transition_end = max(freq - fp.border_width*HEIGHT_FACTOR, 0.);
    let height = smoothstep(border_height_transition_start, border_height_transition_end, pos.y);

    return max(width, height);
}

@fragment
fn fs_color(in: Output) -> @location(0) vec4f {
    return mix(fp.color1, fp.border_color, get_border_mask(in.internal_pos, in.freq));
}

@fragment
fn fs_presence(in: Output) -> @location(0) vec4f {
    let bar_color = mix(fp.color1, fp.color2, smoothstep(0., 1., in.freq));
    return mix(bar_color, fp.border_color, get_border_mask(in.internal_pos, in.freq));
}

@fragment
fn fs_horizontal_gradient(in: Output) -> @location(0) vec4f {
    let bar_color = mix(fp.color1, fp.color2, in.rel_pos.x);
    return mix(bar_color, fp.border_color, get_border_mask(in.internal_pos, in.freq));
}

@fragment
fn fs_vertical_gradient(in: Output) -> @location(0) vec4f {
    let bar_color = mix(fp.color1, fp.color2, smoothstep(0., .8, in.rel_pos.y));
    return mix(bar_color, fp.border_color, get_border_mask(in.internal_pos, in.freq));
}
