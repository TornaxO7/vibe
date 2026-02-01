struct FragmentParams {
    base_color: vec3f,
    time: f32,
    resolution: vec2f,
    movement_speed: f32,
    points_width: u32,
}

@group(0) @binding(0)
var<uniform> fp: FragmentParams;

@group(0) @binding(1)
var<storage, read> points: array<vec2<f32>>;

@group(0) @binding(2)
var<storage, read> zoom_factors: array<f32>;

@group(0) @binding(3)
var<storage, read> random_seeds: array<f32>;

@group(0) @binding(4)
var value_noise_texture: texture_2d<f32>;

@group(0) @binding(5)
var value_noise_sampler: sampler;

@group(0) @binding(6)
var<storage, read> freqs: array<f32>;

const CELL_DIAG: f32 = sqrt(2.);

fn get_point(id: vec2<i32>) -> vec2<f32> {
    let points_len = i32(arrayLength(&points));

    let idx = ((id.x + id.y * i32(fp.points_width)) + points_len) % points_len;

    return points[idx];
}

fn cellular_noise(uv: vec2<f32>, layer_idx: u32, time: f32) -> f32 {
    let points_len = arrayLength(&points);
    let zoom_factor = zoom_factors[layer_idx];
    let random_seed = random_seeds[layer_idx];

    let grid_cell = modf(uv * zoom_factor);
    let id: vec2<i32> = vec2<i32>(grid_cell.whole);
    let gv: vec2<f32> = grid_cell.fract;

    var point = get_point(id);
    var min_d = 2.;
    for (var y = -1; y < 2; y++) {
        for (var x = -1; x < 2; x++) {
            let offset = vec2<i32>(x, y);
            let nid = id + offset;

            point = get_point(nid) * time + random_seed * f32(layer_idx);
            point.x = cos(point.x);
            point.y = sin(point.y);
            point = point * .5 + .5;
            point += vec2<f32>(offset);

            min_d = min(min_d, dot(gv - point, gv - point));
        }
    }

    return sqrt(min_d);
}

fn dust_layer(uv: vec2<f32>, color: vec3<f32>) -> vec4<f32> {
    let dust_presence = textureSample(value_noise_texture, value_noise_sampler, uv).r;
    return vec4<f32>(color * dust_presence, 1.);
}

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let time = fp.time;
    var col: vec4<f32>;
    var uv: vec2<f32> = (2. * pos.xy - fp.resolution.xy) / fp.resolution.y;

    let phase = time * fp.movement_speed;
    uv += 10. * vec2f(cos(phase), sin(phase)) + 20.;

    var base_color2: vec3<f32>;
    base_color2.r = cos(time + uv.x + fp.base_color.r);
    base_color2.g = sin(time + uv.y + fp.base_color.g);
    base_color2.b = sin(time * .5 + uv.x + uv.y + fp.base_color.b);
    base_color2 = base_color2 * .1 + .5;

    let amount_layers = arrayLength(&zoom_factors);
    for (var layer_idx: u32 = 0; layer_idx < amount_layers; layer_idx++) {
        var noise_value = cellular_noise(uv, layer_idx, time);

        let x = noise_value;
        var y = smoothstep(.3, CELL_DIAG, x);
        // don't let y become bigger than 1
        y /= f32(amount_layers);

        let freq = freqs[amount_layers - layer_idx - 1];
        col += vec4(base_color2 * y * max(freq * f32(amount_layers), .5), noise_value);
    }

    col += dust_layer(uv, base_color2);
    // don't flash the user, lol
    col = clamp(col, vec4<f32>(0.), vec4<f32>(1.));
    return col;
}