@group(0) @binding(0)
var<uniform> octaves: u32;

@group(0) @binding(1)
var<uniform> seed: f32;

@group(0) @binding(2)
var<uniform> brightness: f32;

@group(0) @binding(3)
var<uniform> canvas_size: vec2<f32>;

// https://www.shadertoy.com/view/4djSRW
fn hash12(p: vec2<f32>) -> f32 {
	var p3 = fract(vec3<f32>(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + seed);
    return fract((p3.x + p3.y) * p3.z) * brightness;
}

fn value_noise(uv: vec2<f32>) -> f32 {
    let split = modf(uv);
    let id: vec2<f32> = split.whole;
    let gv: vec2<f32> = split.fract;

    let tl: f32 = hash12(id + vec2(0., 0.));
    let tr: f32 = hash12(id + vec2(1., 0.));
    let bl: f32 = hash12(id + vec2(0., 1.));
    let br: f32 = hash12(id + vec2(1., 1.));

    let sx = smoothstep(0., 1., gv.x);
    let sy = smoothstep(0., 1., gv.y);

    let w1 = mix(tl, tr, sx);
    let w2 = mix(bl, br, sx);
    return mix(w1, w2, sy);
}

const GAMMA: f32 = 2.2;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var presence: f32 = 0.;
    var uv = pos.xy / canvas_size.xy;
    uv.x *= canvas_size.x / canvas_size.y;

    for (var i: u32 = 0; i < octaves; i++) {
        presence += value_noise(uv * pow(2., f32(i))) * pow(2., -f32(i));
    }

    var col: vec3<f32> = vec3<f32>(presence);
    col.r = pow(col.r, GAMMA);
    col.g = pow(col.g, GAMMA);
    col.b = pow(col.b, GAMMA);
    return vec4<f32>(col, 1.);
}
