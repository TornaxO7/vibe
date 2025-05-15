@group(0) @binding(0)
var<storage, read> bar_rotation: array<mat2x2f>;

@group(0) @binding(1)
var<storage, read> inverse_bar_rotation: array<mat2x2f>;

@group(0) @binding(2)
var<uniform> bar_width: f32;

@group(0) @binding(3)
var<uniform> circle_radius: f32;

@group(0) @binding(4)
var<uniform> iResolution: vec2f;

@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

struct Input {
    @builtin(instance_index) instance_idx: u32,
    @builtin(vertex_index) vertex_idx: u32,
};

struct Output {
    @builtin(position) pos: vec4f,
};

@vertex
fn vertex_main(in: Input) -> Output {
    let width: f32 = bar_width / 2.;
    let height: f32 = 0.5 * freqs[in.instance_idx] + circle_radius;
    var pos: vec2f;

    if (in.vertex_idx == 0) {
        pos = vec2f(circle_radius, width);
    } else if (in.vertex_idx == 1) {
        pos = vec2f(circle_radius, -width);
    } else if (in.vertex_idx == 2) {
        pos = vec2f(height, width);
    } else { // in.vertex_idx == 3
        pos = vec2f(height, -width);
    }

    pos = bar_rotation[in.instance_idx] * pos;
    var out: Output;

    pos.x /= iResolution.x / iResolution.y;
    out.pos = vec4f(pos, 0., 1.);

    return out;
}

@vertex
fn vertex_main_inverted(in: Input) -> Output {
    let width: f32 = bar_width / 2.;
    let height: f32 = 0.5 * freqs[in.instance_idx] + circle_radius;
    var pos: vec2f;

    if (in.vertex_idx == 0) {
        pos = vec2f(circle_radius, width);
    } else if (in.vertex_idx == 1) {
        pos = vec2f(circle_radius, -width);
    } else if (in.vertex_idx == 2) {
        pos = vec2f(height, width);
    } else { // in.vertex_idx == 3
        pos = vec2f(height, -width);
    }

    pos = inverse_bar_rotation[in.instance_idx] * pos;
    var out: Output;

    pos.x /= iResolution.x / iResolution.y;
    out.pos = vec4f(pos, 0., 1.);

    return out;
}

@fragment
fn fragment_main(in: Output) -> @location(0) vec4f {
    let col = vec3f(255., 204., 51.) / 255.;
    return vec4f(col, 1.);
}