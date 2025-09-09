@group(0) @binding(0)
var<uniform> seed: f32;

@vertex
fn main_vs(@location(0) pos: vec2f) -> @builtin(position) vec4f {
    return vec4f(pos, 0., 1.);
}

// == fragment ==
fn hash12(p: vec2f) -> f32
{
    var p3  = fract(vec3f(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + vec3f(33.33+seed));
    return fract((p3.x + p3.y) * p3.z);
}

@fragment
fn main_fs(@builtin(position) pos: vec4f) -> @location(0) f32 {
    return hash12(pos.xy);
}