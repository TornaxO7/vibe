@group(0) @binding(0)
var src: texture_storage_2d<r32float, read>;

@group(0) @binding(1)
var dst: texture_storage_2d<r32float, write>;

// https://en.wikipedia.org/wiki/Kernel_(image_processing)
const kernel: array<array<f32, 3>, 3> = array(
    array( 0., -1.,  0.),
    array(-1.,  5., -1.),
    array( 0., -1.,  0.),
);

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    var value: f32 = 0.;
    if (kernel_in_texture(gid.xy)) {
        let igid = vec2i(gid.xy);
        value = apply_kernel(igid);
    }

    textureStore(dst, gid.xy, vec4f(value, 0., 0., 0.));
}

fn apply_kernel(igid: vec2i) -> f32 {
    var sum: f32 = 0.;
    for (var y: i32 = -1; y < 2; y++) {
        for (var x: i32 = -1; x < 2; x++) {
            let coord = igid + vec2i(x, y);

            sum += kernel[y + 1][x + 1] * textureLoad(src, coord).r;
        }
    }

    return sum;
}

fn kernel_in_texture(center: vec2u) -> bool {
    let size = textureDimensions(src);
    
    let x_is_in_texture = 0 < center.x && center.x < size.x - 1;
    let y_is_in_texture = 0 < center.y && center.y < size.y - 1;
    
    return x_is_in_texture && y_is_in_texture;
}