@group(0) @binding(0)
var<uniform> iResolution: vec2f;

@group(0) @binding(1)
var s: sampler;

@group(0) @binding(2)
var t: texture_2d<f32>;

@vertex
fn main_vs(@location(0) pos: vec2f) -> @builtin(position) vec4f {
    return vec4f(pos, 0., 1.);
}

@fragment
fn main_fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    var uv = (2. * pos.xy - iResolution.xy) / iResolution.y;
    uv.y *= -1.;

    let m = textureSample(t, s, uv).r;
    return vec4f(vec3f(m), 1.);
}