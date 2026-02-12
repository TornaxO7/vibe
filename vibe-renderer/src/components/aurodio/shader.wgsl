struct FragmentParams {
    base_color: vec3f,
    time: f32,
    resolution: vec2f,
    movement_speed: f32,
}

@group(0) @binding(0)
var<uniform> fp: FragmentParams;

@group(0) @binding(1)
var<storage, read> zoom_factors: array<f32>;

@group(0) @binding(2)
var value_noise_texture: texture_2d<f32>;

@group(0) @binding(3)
var sampler_nearest: sampler;

@group(0) @binding(4)
var<storage, read> freqs: array<f32>;

const CELL_DIAG: f32 = sqrt(2.);

// Credits: Dave Hoskins
// Source: https://www.shadertoy.com/view/4djSRW
fn hash22(p: vec2f) -> vec2f
{
	var p3 = fract(vec3f(p.xyx) * vec3f(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}

fn cellular_noise(uv: vec2<f32>, layer_idx: u32, time: f32) -> f32 {
    let zoom_factor = zoom_factors[layer_idx];

    let zv = uv * zoom_factor;

    let s = vec2f(1.);
    let id = round(zv / s);
    let gv = zv - s*id;

    var min_d = 3.;
    for (var y = -1; y < 2; y++) {
        for (var x = -1; x < 2; x++) {
            let offset = vec2f(vec2i(x, y));
            let nid = id + offset;

            let h = hash22(vec2f(nid)) * (f32(layer_idx) + 1.) * 100. + fp.time;
            var point = vec2f(cos(h.x), sin(h.y))*.4;
            point += vec2f(offset);

            min_d = min(min_d, dot(gv - point, gv - point));
        }
    }

    return sqrt(min_d);
}

fn dust_layer(uv: vec2<f32>, color: vec3<f32>) -> vec4<f32> {
    let dust_presence = textureSample(value_noise_texture, sampler_nearest, uv).r;
    return vec4<f32>(color * dust_presence, 1.);
}

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let time = fp.time;
    var col: vec4<f32>;
    var uv: vec2<f32> = (2. * pos.xy - fp.resolution.xy) / fp.resolution.y;

    let phase = time * fp.movement_speed;
    uv += 10. * vec2f(cos(phase), sin(phase)) + 20.;

    var base_color: vec3<f32>;
    base_color.r = cos(time + uv.x + fp.base_color.r);
    base_color.g = sin(time + uv.y + fp.base_color.g);
    base_color.b = sin(time * .5 + uv.x + uv.y + fp.base_color.b);
    base_color = base_color * .1 + .5;

    let amount_layers = arrayLength(&zoom_factors);
    for (var layer_idx: u32 = 0; layer_idx < amount_layers; layer_idx++) {
        var noise_value = cellular_noise(uv, layer_idx, time);

        let x = noise_value;
        var y = smoothstep(.3, CELL_DIAG, x);
        // don't let y become bigger than 1
        y /= f32(amount_layers);

        let freq = freqs[amount_layers - layer_idx - 1];
        col += vec4(base_color * y * max(freq * f32(amount_layers), .5), noise_value);
    }

    col += dust_layer(uv, base_color);
    // don't flash the user, lol
    col = clamp(col, vec4<f32>(0.), vec4<f32>(1.));
    return col;
}