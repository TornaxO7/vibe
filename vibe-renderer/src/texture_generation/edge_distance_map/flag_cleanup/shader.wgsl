@group(0) @binding(0)
var src: texture_storage_2d<r32float, read>;

@group(0) @binding(1)
var dst: texture_storage_2d<r32float, write>;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let value = textureLoad(src, gid.xy).r;

    var dst_value = 1.;
    if (is_edge(value)) {
        dst_value = 1.;
    } else {
        dst_value = 0.;
    }

    textureStore(dst, gid.xy, vec4f(dst_value, .0, .0, 0.));
}

fn is_edge(value: f32) -> bool {
    return value > 0.9;
}
