@group(0) @binding(0)
var src: texture_storage_2d<r32float, read>;

@group(0) @binding(1)
var dst: texture_storage_2d<r32float, write>;

const INF: f32 = 9999.;

@compute
@workgroup_size(16, 16, 1)
fn init_map(@builtin(global_invocation_id) gid: vec3u) {
    let value = textureLoad(src, gid.xy).r;

    if (is_edge(value)) {
        textureStore(dst, gid.xy, vec4f(0.));
    } else {
        textureStore(dst, gid.xy, vec4f(INF, 0., 0., 0.));
    }
}

@compute
@workgroup_size(16, 16, 1)
fn update_dist(@builtin(global_invocation_id) gid: vec3u) {
    let igid = vec2i(gid.xy);

    var min_dist = textureLoad(src, gid.xy).r;
    for (var x: i32 = -1; x < 2; x++) {
        for (var y: i32 = -1; y < 2; y++) {
            let offset = vec2i(x, y);
            let coord = igid + offset;

            if (is_valid_coord(coord)) {
                let neighbour_distance = textureLoad(src, coord).r;

                // we skip "invalid" distances
                if (neighbour_distance < INF) {
                    let new_dist = neighbour_distance + length(vec2f(offset));

                    min_dist = min(min_dist, new_dist);
                }
            }
        }
    }

    textureStore(dst, gid.xy, vec4f(min_dist, 0., 0., 0.));
}

@compute
@workgroup_size(16, 16, 1)
fn normalize_distances(@builtin(global_invocation_id) gid: vec3u) {
    let size = vec2f(textureDimensions(dst));
    let max_dist = max(size.x, size.y);

    let value = textureLoad(src, gid.xy).r;
    textureStore(dst, gid.xy, vec4f(value / max_dist, 0., 0., 0.));
}

fn is_valid_coord(coord: vec2i) -> bool {
    let size = vec2i(textureDimensions(dst));

    let x_is_valid = 0 <= coord.x && coord.x < size.x;
    let y_is_valid = 0 <= coord.y && coord.y < size.y;

    return x_is_valid && y_is_valid;
}

fn is_edge(value: f32) -> bool {
    return value > 0.9;
}
