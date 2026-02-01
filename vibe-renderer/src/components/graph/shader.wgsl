struct VertexParams {
    // vector pointing to the right side of the box in vertex space.
    // - Isn't normalized.
    // - Length of this vector returns the width of the box
    right: vec2f,
    // the bottom left corner of the box in vertex space
    bottom_left_corner: vec2f,
    // vector pointing at the top of the box. Length is also the height of the box.
    // - Isn't normalized
    // - Length of this vector returns the height of the box
    up: vec2f,
}

struct FragmentParams {
    color1: vec4f,
    color2: vec4f,
}

@group(0) @binding(0)
var<uniform> vp: VertexParams;

@group(0) @binding(1)
var<uniform> fp: FragmentParams;

// used in fragment shader
@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

struct Input {
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32,
}

struct Output {
    @builtin(position) pos: vec4f,
    // the relative position within the box
    @location(0) rel_pos: vec2f,
}

// Assuming:
//
//   y (up)
// ^
// |  0    1
// |   ----
// |   |\ |
// |   | \|
// |   ----
// |  2    3
// |
// ------> x (right)
@vertex
fn bass_treble(in: Input) -> Output {
    return bass_treble_inner(in);
}

@vertex
fn treble_bass(in: Input) -> Output {
    var out = bass_treble_inner(in);

    // flip the relative x coords so that the correct frequency is chosen in the
    // fragment shader
    out.rel_pos.x = 1. - out.rel_pos.x;
    return out;
}

// for whatever reason, you can't call an entrypoint function... so the `bass_treble` logic goes here
fn bass_treble_inner(in: Input) -> Output {
    // move further to the right for the second block.
    // Note: `* 0.98` due to floating point issues. Move the right block a bit to the left block so that there will be no gap.
    var pos = vp.bottom_left_corner + (vp.right * 0.98) * f32(in.instance_idx);
    var rel_pos = vec2f(0.);

    let is_top = in.vertex_idx <= 1;
    if (is_top) {
        pos += vp.up;
        rel_pos.y = 1.;
    }

    let is_right = in.vertex_idx == 1 || in.vertex_idx == 3;
    if (is_right) {
        pos += vp.right;
        rel_pos.x = 1.;
    }

    var out: Output;
    out.pos = vec4f(pos, 0., 1.);
    out.rel_pos = rel_pos;
    return out;
}

// == fragment code ==
fn get_mask(rel_pos: vec2f) -> f32 {
    var freq = freqs[u32(floor(rel_pos.x * f32(arrayLength(&freqs))))]*.9;
    freq = max(freq, 1e-5);

    let top = smoothstep(freq, freq*.9, rel_pos.y);
    let bottom = smoothstep(.0, .01, rel_pos.y);
    return bottom * top;
}

@fragment
fn color(in: Output) -> @location(0) vec4f {
    return fp.color1 * get_mask(in.rel_pos);
}

@fragment
fn horizontal_gradient(in: Output) -> @location(0) vec4f {
    let mask = get_mask(in.rel_pos);
    let col = mix(fp.color1, fp.color2, in.rel_pos.x);
    return col * mask;
}

@fragment
fn vertical_gradient(in: Output) -> @location(0) vec4f {
    let mask = get_mask(in.rel_pos);
    let col = mix(fp.color1, fp.color2, smoothstep(0., 1., in.rel_pos.y));
    return col * mask;
}