@group(0) @binding(0)
var<uniform> octaves: u32;

@group(0) @binding(1)
var white_noise_texture: texture_2d<f32>;

@group(0) @binding(2)
var white_noise_sampler: sampler;

@group(0) @binding(3)
var<uniform> canvas_size: f32;

fn hash(id: vec2f) -> f32 {
    return textureSample(white_noise_texture, white_noise_sampler, id).r;
}

fn value_noise(uv: vec2<f32>) -> f32 {
    let split = modf(uv);
    let id: vec2<f32> = split.whole;
    let gv: vec2<f32> = split.fract;

    let tl: f32 = hash(id + vec2f(0., 0.) + gv);
    let tr: f32 = hash(id + vec2f(1., 0.) + gv);
    let bl: f32 = hash(id + vec2f(0., 1.) + gv);
    let br: f32 = hash(id + vec2f(1., 1.) + gv);

    let sx = smoothstep(0., 1., gv.x);
    let sy = smoothstep(0., 1., gv.y);

    let w1 = mix(tl, tr, sx);
    let w2 = mix(bl, br, sx);
    return mix(w1, w2, sy);
}

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) f32 {
    var presence: f32 = 0.;
    let uv = pos.xy / canvas_size;

    var freq = 1.;
    var amp = .5;
    for (var i: u32 = 0; i < octaves; i++) {
        presence += amp * value_noise(freq * uv);

        freq *= 2.;
        amp *= .5;
    }

    return presence;
}
