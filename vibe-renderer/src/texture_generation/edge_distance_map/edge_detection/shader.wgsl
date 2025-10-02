@group(0) @binding(0)
var src: texture_storage_2d<r32float, read>;

@group(0) @binding(1)
var dst: texture_storage_2d<rg32float, write>;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    var value = vec2f(0.);
    if (kernels_within_texture(gid.xy)) {
        let v = compute_vertical_value(gid.xy);
        let h = compute_horizontal_value(gid.xy);

        let magnitude = clamp(sqrt(v*v + h*h), 0., 1.);
        let radian = atan2(v, h);
        value = vec2f(magnitude, radian);
    }

    textureStore(dst, gid.xy, vec4f(value, 0., 1.));
}

fn compute_vertical_value(coord: vec2u) -> f32 {
    const kernel: array<array<f32, 3>, 3> = array(
        array( 1.,  2.,  1.),
        array( 0.,  0.,  0.),
        array(-1., -2., -1.),
    );

    return apply_kernel(coord, kernel);
}

fn compute_horizontal_value(coord: vec2u) -> f32 {
    const kernel: array<array<f32, 3>, 3> = array(
        array(-1., 0., 1.),
        array(-2., 0., 2.),
        array(-1., 0., 1.),
    );

    return apply_kernel(coord, kernel);
}

fn apply_kernel(coord: vec2u, kernel: array<array<f32, 3>, 3>) -> f32 {
    let icoord = vec2i(coord);

    var sum: f32 = 0.;
    for (var x: i32 = -1; x < 2; x++) {
        for (var y: i32 = -1; y < 2; y++) {
            let pos = icoord + vec2i(x, y);
            sum += kernel[y + 1][x + 1] * textureLoad(src, pos).r;
        }
    }

    return sum;
}

// check if the whole kernel with the given center coordinate
// is still within the texture
fn kernels_within_texture(coord: vec2u) -> bool {
    let size = textureDimensions(src);

    let valid_x = 0 < coord.x && coord.x < size.x - 1;
    let valid_y = 0 < coord.y && coord.y < size.y - 1;

    return valid_x && valid_y;
}
