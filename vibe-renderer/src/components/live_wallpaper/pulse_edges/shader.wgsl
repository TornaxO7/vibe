@group(0) @binding(0)
var wallpaper: texture_2d<f32>;

@group(0) @binding(1)
var edges: texture_2d<f32>;

@group(0) @binding(2)
var blurred_wallpaper: texture_2d<f32>;

@group(0) @binding(3)
var sam: sampler;

struct Data {
    resolution: vec2f,
    time: f32,
    freq: f32,

    wallpaper_brightness: f32,
    edge_width: f32,
    pulse_brightness: f32,
}

@group(0) @binding(4)
var<uniform> data: Data;

@vertex
fn vs_main(@location(0) pos: vec2f) -> @builtin(position) vec4f {
    return vec4f(pos, 0., 1.);
}

@fragment
fn fs_main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv = pos.xy / data.resolution;

    let tex = textureSample(wallpaper, sam, uv);
    let dis = textureSample(edges, sam, uv).r;
    let blur = textureSample(blurred_wallpaper, sam, uv);

    const MAX_FREQ: f32 = .4;
    let freq = clamp(data.freq*MAX_FREQ, 1e-3, MAX_FREQ);

    const MIN_PRESENCE: f32 = 1.;
    let m = data.pulse_brightness / max(data.edge_width*dis, MIN_PRESENCE);

    return m*blur*freq + vec4f(tex.rgb*data.wallpaper_brightness, 1.);
}
