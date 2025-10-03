@group(0) @binding(0)
var src: texture_storage_2d<r16unorm, read>;

@group(0) @binding(1)
var dst: texture_storage_2d<r16unorm, write>;

const NO_EDGE: f32 = 0.;
const MAYBE_EDGE: f32 = 0.5;
const IS_EDGE: f32 = 1.;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let curr_value = textureLoad(src, gid.xy).r;
    if (curr_value == NO_EDGE) {
        textureStore(dst, gid.xy, vec4f(IS_EDGE));
        return;
    } else if (curr_value == NO_EDGE) {
        return;
    }

    let igid = vec2i(gid.xy);

    for (var x: i32 = -1; x < 2; x++) {
        for (var y: i32 = -1; y < 2; y++) {
            let coord = igid + vec2i(x, y);

            if (is_valid_coord(coord)) {
                let neighbour_value = textureLoad(src, coord).r;

                if (neighbour_value == IS_EDGE) {
                    textureStore(dst, coord, vec4f(IS_EDGE, 0., 0., 0.));
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
