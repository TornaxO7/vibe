@group(0) @binding(0)
var src: texture_storage_2d<r32float, read>;

@group(0) @binding(1)
var dst: texture_storage_2d<r32float, write>;

const NO_EDGE: f32 = 0.;
const MAYBE_EDGE: f32 = 0.5;
const IS_EDGE: f32 = 1.;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let curr_value = textureLoad(src, gid.xy).r;
    if (is_an_edge(curr_value)) {
        textureStore(dst, gid.xy, vec4f(IS_EDGE, 0., 0., 0.));
        return;
    } else if (is_not_an_edge(curr_value)) {
        textureStore(dst, gid.xy, vec4f(NO_EDGE, 0., 0., 0.));
        return;
    }

    let igid = vec2i(gid.xy);

    const SEARCH_RADIUS: i32 = 4;
    for (var x: i32 = -SEARCH_RADIUS; x <= SEARCH_RADIUS; x++) {
        for (var y: i32 = -SEARCH_RADIUS; y <= SEARCH_RADIUS; y++) {
            let coord = igid + vec2i(x, y);

            if (is_valid_coord(coord)) {
                let neighbour_value = textureLoad(src, coord).r;

                if (is_an_edge(neighbour_value)) {
                    textureStore(dst, gid.xy, vec4f(IS_EDGE, 0., 0., 0.));
                    return;
                }
            }
        }
    }
}

fn is_valid_coord(coord: vec2i) -> bool {
    let size = vec2i(textureDimensions(dst));

    let x_is_valid = 0 <= coord.x && coord.x < size.x;
    let y_is_valid = 0 <= coord.y && coord.y < size.y;

    return x_is_valid && y_is_valid;
}

fn is_an_edge(value: f32) -> bool {
    return value > 0.9;
}

fn is_not_an_edge(value: f32) -> bool {
    return value < 0.1;
}
