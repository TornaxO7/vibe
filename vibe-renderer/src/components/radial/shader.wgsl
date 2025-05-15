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

@group(0) @binding(5)
var<uniform> bar_height_sensitivity: f32;

@group(0) @binding(6)
var<uniform> color: vec4f;

@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

struct Input {
    @builtin(instance_index) instance_idx: u32,
    @builtin(vertex_index) vertex_idx: u32,
};

// TODO: Somehow avoid duplicated code between `vertex_main` and `vertex_main_invertex`
@vertex
fn vertex_main(in: Input) -> @builtin(position) vec4f {
    let width: f32 = bar_width / 2.;
    let height: f32 = bar_height_sensitivity * freqs[in.instance_idx] + circle_radius;
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
    pos.x /= iResolution.x / iResolution.y;
    return vec4f(pos, 0., 1.);
}

@vertex
fn vertex_main_inverted(in: Input) -> @builtin(position) vec4f {
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
    pos.x /= iResolution.x / iResolution.y;
    return vec4f(pos, 0., 1.);
}

@fragment
fn fragment_main() -> @location(0) vec4f {
    return color;
}