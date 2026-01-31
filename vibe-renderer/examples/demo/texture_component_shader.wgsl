const VERTICES: array<vec2f, 3> = array(
    vec2f(-3., -1.), // bottom left
    vec2f(1., -1.), // bottom right
    vec2f(1., 3.) // top right
);

struct FragmentParams {
    resolution: vec2f,
}

@group(0) @binding(0)
var<uniform> fp: FragmentParams;

@group(0) @binding(1)
var s: sampler;

@group(0) @binding(2)
var t: texture_2d<f32>;

@vertex
fn main_vs(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4f {
    return vec4f(VERTICES[idx], 0., 1.);
}

@fragment
fn main_fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    var uv = (2. * pos.xy - fp.resolution.xy) / fp.resolution.y;
    uv.y *= -1.;

    let m = textureSample(t, s, uv).r;
    return vec4f(vec3f(m), 1.);
}