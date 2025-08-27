@group(0) @binding(0)
var<storage, read> bar_rotation: array<mat2x2f>;

@group(0) @binding(1)
var<storage, read> inverse_bar_rotation: array<mat2x2f>;

@group(0) @binding(2)
var<uniform> bar_width: f32;

@group(0) @binding(3)
var<uniform> circle_radius: f32;

@group(0) @binding(4)
var<uniform> iResolution: vec3f;

@group(0) @binding(5)
var<uniform> bar_height_sensitivity: f32;

@group(0) @binding(6)
var<uniform> color: vec4f;

@group(0) @binding(7)
var<uniform> position_offset: vec2f;

@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

struct Input {
    @builtin(instance_index) instance_idx: u32,
    @builtin(vertex_index) vertex_idx: u32,
};

struct Output {
    @builtin(position) vpos: vec4f,
    @location(0) rect_pos: vec2f,
};

@vertex
fn vertex_main(in: Input) -> Output {
    return _inner_vertex_main(in, bar_rotation[in.instance_idx]);
}

@vertex
fn vertex_main_inverted(in: Input) -> Output {
    return _inner_vertex_main(in, inverse_bar_rotation[in.instance_idx]);
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
fn _inner_vertex_main(in: Input, bar_rotation: mat2x2f) -> Output {
    let width: f32 = bar_width / 2.;
    let height: f32 = bar_height_sensitivity * freqs[in.instance_idx] + circle_radius;
    var rect_pos: vec2f;

    if (in.vertex_idx == 0) {
        rect_pos = vec2f(circle_radius, width);
    } else if (in.vertex_idx == 1) {
        rect_pos = vec2f(circle_radius, -width);
    } else if (in.vertex_idx == 2) {
        rect_pos = vec2f(height, width);
    } else { // in.vertex_idx == 3
        rect_pos = vec2f(height, -width);
    }

    var final_pos: vec2f;
    final_pos = bar_rotation * rect_pos;
    final_pos.x /= iResolution.z;
    final_pos += position_offset;

    var out: Output;
    out.vpos = vec4f(final_pos, 0., 1.);
    out.rect_pos = vec2f(rect_pos.x - circle_radius, rect_pos.y);
    return out;
}


@fragment
fn color_entrypoint(in: Output) -> @location(0) vec4f {
    return _fragment_smoothing(color, in.rect_pos);
}

@group(0) @binding(8)
var<uniform> height_gradient_inner: vec4f;

@group(0) @binding(9)
var<uniform> height_gradient_outer: vec4f;

@fragment
fn height_gradient_entrypoint(in: Output) -> @location(0) vec4f {
    let col = mix(height_gradient_inner, height_gradient_outer, smoothstep(0., .5, in.rect_pos.x / bar_height_sensitivity));
    return _fragment_smoothing(col, in.rect_pos);
}

fn _fragment_smoothing(col: vec4f, rect_pos: vec2f) -> vec4f {
    
    let max_width = bar_width / 2.;
    let rel_y = abs(rect_pos.y) / max_width;

    let width_smoothing = smoothstep(1., 0., rel_y);

    // smooth out the line at the edge of the inner circle
    let bottom_smoothing = smoothstep(.0, .01, rect_pos.x);
    return col * width_smoothing * bottom_smoothing;
}