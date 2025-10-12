@group(0) @binding(0)
var<storage, read> max_value: u32;

struct Ratios {
    upper: f32,
    lower: f32,
}

@group(0) @binding(1)
var<uniform> ratios: Ratios;

@group(0) @binding(2)
var src: texture_storage_2d<r32float, read>;

@group(0) @binding(3)
var dst: texture_storage_2d<r32float, write>;

const NO_EDGE: f32 = 0.;
const MAYBE_EDGE: f32 = 0.5;
const IS_EDGE: f32 = 1.;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let value = textureLoad(src, gid.xy).r;

    let high_threshold = bitcast<f32>(max_value) * ratios.upper;
    let low_threshold = high_threshold * ratios.lower;

    var dst_value = NO_EDGE;
    if (low_threshold < value) {
        dst_value = MAYBE_EDGE;
    }
    if (high_threshold < value) {
        dst_value = IS_EDGE;
    }

    textureStore(dst, gid.xy, vec4f(dst_value, 0., 0., 0.));
}
