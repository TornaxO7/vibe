struct VertexParams {
    canvas_height: f32,
}

struct FragmentParams {
    color1: vec4f,
    time: f32,
}

@group(0) @binding(0)
var<uniform> vp: VertexParams;

@group(0) @binding(1)
var<uniform> fp: FragmentParams;

struct Input {
    @builtin(vertex_index) vertex_idx: u32,
}

struct Output {
    @builtin(position) pos: vec4f,
    @location(0) rel: vec2f,
}

//            (top)
//        0          2
//        |---------|
// (left) |         | (right)
//        |---------|
//        1          3
//          (bottom)
@vertex
fn vs_main(in: Input) -> Output {
    var out: Output;
    var pos: vec2f;

    // x
    let is_left = in.vertex_idx <= 1;
    if (is_left) {
        out.rel.x = 0.;
        pos.x = -1.;
    } else {
        out.rel.x = 1.;
        pos.x = 1.;
    }

    // y
    let is_top = in.vertex_idx == 0 || in.vertex_idx == 2;
    if (is_top) {
        out.rel.y = 1.;
        pos.y = 2. * vp.canvas_height - 1.;
    } else {
        out.rel.y = 0.;
        pos.y = -1.;
    }

    out.pos = vec4f(pos, 0., 1.);
    
    return out;
}

@fragment
fn fs_main(in: Output) -> @location(0) vec4f {
    const PI: f32 = acos(-1.);
    let wave = sin(fp.time+2.*PI*in.rel.x)*.025;

    let b = 0.09;
    let a = .1 / (in.rel.y + b + wave) - b;
    return vec4f(fp.color1.rgb, min(fp.color1.a, a));
}
