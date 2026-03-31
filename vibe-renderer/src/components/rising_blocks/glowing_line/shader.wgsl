struct FragmentParams {
    time: f32,
}

@group(0) @binding(0)
var<uniform> fp: FragmentParams;

struct Input {
    @builtin(vertex_index) vertex_idx: u32,
}

struct Output {
    @builtin(position) pos: vec4f,
    @location(0) rel_p: vec2f,
}

// 0           2
//  |---------|
//  |         |
//  |---------|
// 1           3
const VERTICES: array<vec2f, 4> = array(
    vec2f(-1., 0.), // top left
    vec2f(-1., -1.0), // bottom left
    vec2f(1., 0.), // top right
    vec2f(1., -1.0), // bottom right
);

@vertex
fn vs_main(in: Input) -> Output {
    var out: Output;

    out.pos = vec4f(VERTICES[in.vertex_idx], 0., 1.);

    // x
    let is_left = in.vertex_idx <= 1;
    if (is_left) {
        out.rel_p.x = 0.;
    } else {
        out.rel_p.x = 1.;
    }

    // y
    let is_top = in.vertex_idx == 0 || in.vertex_idx == 2;
    if (is_top) {
        out.rel_p.y = 1.;
    } else {
        out.rel_p.y = 0.;
    }
    
    return out;
}

@fragment
fn fs_main(in: Output) -> @location(0) vec4f {
    const PI: f32 = acos(-1.);
    let a = .1 / (in.rel_p.y + .1 + sin(fp.time+2.*PI*in.rel_p.x)*.025) - .1;
    return vec4f(vec3f(0., 1., 1.), a);
}
