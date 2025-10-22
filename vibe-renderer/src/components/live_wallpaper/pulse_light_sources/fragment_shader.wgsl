@group(0) @binding(0)
var wallpaper: texture_2d<f32>;

@group(0) @binding(1)
var distance_map: texture_2d<f32>;

@group(0) @binding(2)
var s: sampler;

struct Data {
    resolution: vec2f,
    wallpaper_brightness: f32,
}

@group(0) @binding(3)
var<uniform> data: Data;

@fragment
fn main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv = pos.xy / data.resolution;

    let wallpaper_texel = textureSample(wallpaper, s, uv);
    let dist = textureSample(distance_map, s, uv).rg;

    let p = 1. / (.01*dist.x);

    return wallpaper_texel*data.wallpaper_brightness + wallpaper_texel*p;
}