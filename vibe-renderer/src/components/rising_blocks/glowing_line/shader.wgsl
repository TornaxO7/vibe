struct Input {
    @builtin(vertex_index) vertex_idx: u32,
}

struct Output {
    @builtin(position) pos: vec4f,
    @location(0) rel_y: f32,
}

// 0           2
//  |---------|
//  |         |
//  |---------|
// 1           3
const VERTICES: array<vec2f, 4> = array(
    vec2f(-1., -0.9), // top left
    vec2f(-1., -1.1), // bottom left
    vec2f(1., -0.9), // top right
    vec2f(1., -1.1), // bottom right
);

@vertex
fn vs_main(in: Input) -> Output {
    var out: Output;

    out.pos = vec4f(VERTICES[in.vertex_idx], 0., 1.);

    let is_top = in.vertex_idx == 0 || in.vertex_idx == 2;
    if (is_top) {
        out.rel_y = 1.;
    } else {
        out.rel_y = -1.;
    }
    
    return out;
}

@fragment
fn fs_main(in: Output) -> @location(0) vec4f {
    let y = abs(in.rel_y);
    let a = smoothstep(1., 0., y);
    let col = vec3f(1., 0., 0.) * a;
    return vec4f(col, clamp(a, 0., 1.));
}
