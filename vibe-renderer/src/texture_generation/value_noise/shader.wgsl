@group(0) @binding(0)
var output: texture_storage_2d<r16unorm, write>;

struct Data {
    octaves: u32,
    seed: f32,
    canvas_size: f32,
}

@group(0) @binding(1)
var<uniform> data: Data;

// https://www.shadertoy.com/view/4djSRW
fn hash12(p: vec2<f32>) -> f32 {
	var p3 = fract(vec3<f32>(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx+33.33+ data.seed);
    return fract((p3.x + p3.y) * p3.z);
}

fn value_noise(uv: vec2<f32>) -> f32 {
    let split = modf(uv);
    let id: vec2<f32> = split.whole;
    let gv: vec2<f32> = split.fract;

    let tl: f32 = hash12(id + vec2f(0., 0.));
    let tr: f32 = hash12(id + vec2f(1., 0.));
    let bl: f32 = hash12(id + vec2f(0., 1.));
    let br: f32 = hash12(id + vec2f(1., 1.));
    
    let sx = smoothstep(0., 1., gv.x);
    let sy = smoothstep(0., 1., gv.y);

    let w1 = mix(tl, tr, sx);
    let w2 = mix(bl, br, sx);
    return mix(w1, w2, sy);
}

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    var presence: f32 = 0.;
    let uv = vec2f(gid.xy) / data.canvas_size + data.seed;

    var freq = 1.;
    var amp = .5;
    for (var i: u32 = 0; i < data.octaves; i++) {
        presence += amp * value_noise(freq * uv);

        freq *= 2.;
        amp *= .5;
    }

    textureStore(output, gid.xy, vec4f(presence, 0., 0., 1.));
}
