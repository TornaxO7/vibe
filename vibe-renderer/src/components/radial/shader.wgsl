struct VertexParams {
    position_offset: vec2f,   
    circle_radius: f32,
    aspect_ratio: f32,
}

struct FragmentParams {
    color1: vec4f,
    color2: vec4f,
}

struct VertexFragmentParams {
    bar_width: f32,
    bar_height_sensitivity: f32,
}

@group(0) @binding(0)
var<uniform> vp: VertexParams;

@group(0) @binding(1)
var<uniform> fp: FragmentParams;

@group(0) @binding(2)
var<uniform> vfp: VertexFragmentParams;

@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

@group(1) @binding(1)
var<storage, read> rotations: array<mat2x2f>;

struct Input {
    @builtin(instance_index) instance_idx: u32,
    @builtin(vertex_index) vertex_idx: u32,
};

struct Output {
    @builtin(position) vpos: vec4f,
    @location(0) rect_pos: vec2f,
};

@vertex
fn bass_treble(in: Input) -> Output {
    let freq = freqs[in.instance_idx];
    return vertex_main(freq, in.vertex_idx, in.instance_idx);
}

@vertex
fn treble_bass(in: Input) -> Output {
    let freq = freqs[arrayLength(&freqs) - 1 - in.instance_idx];
    return vertex_main(freq, in.vertex_idx, in.instance_idx);
}

// Convention: We construct the bar where the tip of the bar is pointing to the right.
//
//   y
//   ^
//   | 2     0
//   |  -----                -
//   |  |\  |                |
//   |--|-\-|--> x   -       | bar_width
//   |  |  \|        | width |
//   |  -----        -       -
//   | 3     1
//   |
//   |-----|
//   height
//
fn vertex_main(freq: f32, vertex_idx: u32, instance_idx: u32) -> Output {
    let width: f32 = vfp.bar_width / 2.;
    let height: f32 = vfp.bar_height_sensitivity * freq + vp.circle_radius;
    var rect_pos: vec2f;

    if (vertex_idx == 0) {
        rect_pos = vec2f(vp.circle_radius, width);
    } else if (vertex_idx == 1) {
        rect_pos = vec2f(vp.circle_radius, -width);
    } else if (vertex_idx == 2) {
        rect_pos = vec2f(height, width);
    } else { // in.vertex_idx == 3
        rect_pos = vec2f(height, -width);
    }

    var final_pos: vec2f;
    final_pos = rotations[instance_idx] * rect_pos;
    final_pos.x /= vp.aspect_ratio;
    final_pos += vp.position_offset;

    var out: Output;
    out.vpos = vec4f(final_pos, 0., 1.);
    out.rect_pos = vec2f(rect_pos.x - vp.circle_radius, rect_pos.y);
    return out;
}

// == fragment stuff ==
@fragment
fn fs_color(in: Output) -> @location(0) vec4f {
    return _fragment_smoothing(fp.color1, in.rect_pos);
}

@fragment
fn fs_height_gradient(in: Output) -> @location(0) vec4f {
    let col = mix(fp.color1, fp.color2, smoothstep(0., .5, in.rect_pos.x / vfp.bar_height_sensitivity));
    return _fragment_smoothing(col, in.rect_pos);
}

fn _fragment_smoothing(col: vec4f, rect_pos: vec2f) -> vec4f {
    let max_width = vfp.bar_width / 2.;
    let rel_y = abs(rect_pos.y) / max_width;

    let width_smoothing = smoothstep(1., 0., rel_y);

    // smooth out the line at the edge of the inner circle
    let bottom_smoothing = smoothstep(.0, .01, rect_pos.x);
    return col * width_smoothing * bottom_smoothing;
}