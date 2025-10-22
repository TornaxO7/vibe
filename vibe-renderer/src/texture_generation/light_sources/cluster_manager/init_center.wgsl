@group(0) @binding(0)
var img: texture_storage_2d<r32float, read>;

@group(0) @binding(1)
var distance_map: texture_storage_2d<rg32float, write>;

@group(0) @binding(2)
var<uniform> random_point: vec2u;

const F32_MAX: f32 = 3.4028235e38;
const FIRST_CLUSTER: f32 = 0.;

@compute
@workgroup_size(16, 16, 1)
fn (@builtin(global_invocation_id) gid: vec3u) {
    let texel = textureLoad(img, gid.xy).r;
    if (!is_light(texel)) {
        textureStore(distance_map, gid.xy, vec4f(F32_MAX, FIRST_CLUSTER, 0., 0.));
        return;
    }

    
}

fn is_light(texel: f32) -> bool {
    return texel > 0.5;
}
