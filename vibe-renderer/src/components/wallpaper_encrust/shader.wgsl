@group(0) @binding(0)
var wallpaper: texture_2d<f32>;

@group(0) @binding(1)
var edges: texture_2d<f32>;

@group(0) @binding(2)
var sam: sampler;

struct Data {
    resolution: vec2f,
    time: f32,
}

@group(0) @binding(3)
var<uniform> data: Data;

// @group(0) @binding(3)
// var<storage, read> freqs: array<f32>;

@vertex
fn vs_main(@location(0) pos: vec2f) -> @builtin(position) vec4f {
    return vec4f(pos, 0., 1.);
}

@fragment
fn fs_main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv = pos.xy / data.resolution;

    let tex = textureSample(wallpaper, sam, uv);
    let dis = textureSample(edges, sam, uv).r;

    let m = 1. / (dis * 13. + 1.);

    let col = .2 * tex.rgb + m * vec3f(1.) * (sin(data.time)*.2 + .2);
    return vec4f(col, tex.a);
}
