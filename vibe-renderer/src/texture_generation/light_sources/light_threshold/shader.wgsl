@group(0) @binding(0)
var src: texture_storage_2d<r32float, read>;

@group(0) @binding(1)
var dst: texture_storage_2d<r32float, write>;

@group(0) @binding(2)
var<uniform> threshold: f32;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let texel = textureLoad(src, gid.xy).r;

    let is_not_light_source = texel <= threshold;
    if (is_not_light_source) {
        textureStore(dst, gid.xy, vec4f(0., 0., 0., 0.));
    } else {
        textureStore(dst, gid.xy, vec4f(1., 0., 0., 0.));
    }
}