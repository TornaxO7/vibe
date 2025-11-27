struct GeneralData {
    resolution: vec2f,
}

@group(0) @binding(0)
var<uniform> general_data: GeneralData;

@group(0) @binding(1)
var wallpaper: texture_2d<f32>;

@group(0) @binding(2)
var s: sampler;

struct LightData {
    center: vec2f,
    freq: f32,
    radius: f32
}

@group(0) @binding(3)
var<storage, read> light_datas: array<LightData>;

@fragment
fn main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv = pos.xy / general_data.resolution.xy;
    let texel = textureSample(wallpaper, s, uv);

    var result = vec4f(0.);
    for (var i: u32 = 0; i < arrayLength(&light_datas); i++) {
        let light_data = light_datas[i];

        var center = uv - light_data.center;
        // fix aspect ratio
        center.x /= general_data.resolution.y / general_data.resolution.x;

        let flashbang_protection = min(light_data.freq * .075, .1);
        let x = length(center)*light_data.radius + .1 - max(flashbang_protection, 1e-3);
        let presence = .1 / max(x, 1e-3) - .1;

        result += presence * texel;
    }

    return result;
}

const GREEN: vec4f = vec4f(0., 1., 0., 1.);

@fragment
fn debug(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv = pos.xy / general_data.resolution.xy;
    let texel = textureSample(wallpaper, s, uv);

    var result = vec4f(0.);
    let light_datas_len = arrayLength(&light_datas);
    for (var i: u32 = 0; i < light_datas_len; i++) {
        let light_data = light_datas[i];

        var center = uv - light_data.center;
        center.x /= general_data.resolution.y / general_data.resolution.x;

        // TODO: Circle is not correctly shown
        let x = length(center);
        if (x >= .01) {
            result += texel / f32(light_datas_len);
        } else {
            result += GREEN;
        }
    }

    return result;
}