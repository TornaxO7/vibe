@group(0) @binding(0)
var<uniform> iResolution: vec2<f32>;

@group(0) @binding(1)
var<uniform> movement_speed: f32;

@group(0) @binding(2)
var<uniform> zoom_factor: f32;

@group(0) @binding(3)
var grid_texture: texture_2d<f32>;

@group(0) @binding(4)
var grid_sampler: sampler;

@group(1) @binding(0)
var<uniform> iTime: f32;

@group(1) @binding(1)
var<storage, read> freqs: array<f32>;

fn hash12(p: vec2f) -> f32
{
	var p3  = fract(vec3f(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}


fn get_freq(hash: f32) -> f32 {
    let max_idx = f32(arrayLength(&freqs));
    let idx = u32(floor(hash * max_idx));
    return freqs[idx];
}

@fragment
fn main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    var uv = (2. * pos.xy - iResolution.xy) / iResolution.y;
    let phase = iTime * movement_speed;
    uv += 5. * vec2f(cos(phase), sin(phase));

    let hash = hash12(floor(uv * zoom_factor));
    let freq = exp(get_freq(hash)) - 1.;

    let cell_presence: f32 = textureSample(grid_texture, grid_sampler, fract(uv * zoom_factor)).r;

    let base_color = sin(2. * vec3f(1., 2., 3.) + hash + iTime * .5) * .4 + .6;
    let col = base_color * freq * cell_presence;
    return vec4f(col, 1.);
}