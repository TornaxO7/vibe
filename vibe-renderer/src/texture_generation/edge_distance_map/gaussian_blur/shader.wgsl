@group(0) @binding(0)
var src: texture_storage_2d<r16unorm, read>;

@group(0) @binding(1)
var dst: texture_storage_2d<r16unorm, write>;

@group(0) @binding(2)
var<storage, read> kernel: array<f32>;

@compute
@workgroup_size(16, 16, 1)
fn horizontal(@builtin(global_invocation_id) gid: vec3u) {
    let igid = vec2i(gid.xy);

    let kernel_size: i32 = i32(floor(sqrt(arrayLength(&kernel))));
    let half_kernel_size: i32 = kernel_size / 2;

    let src_size: vec2i = vec2i(textureDimensions(src));

    var sum: f32 = 0.;
    for (var x = -half_kernel_size; x <= half_kernel_size; x++) {
        let coord = igid + vec2i(x, 0.);

        if (coord.x < 0 || coord.x >= src_size.x) {
            return;
        }

        sum += kernel[x + half_kernel_size] * textureLoad(src, coord, 0);
    }

    sum = clamp(0., 1., sum);
    textureStore(dst, gid, vec4f(sum, 0., 0., 0.));
}

@compute
@workgroup_size(16, 16, 1)
fn vertical(@builtin(global_invocation_id) gid: vec3u) {
    let igid = vec2i(gid.xy);

    let kernel_size: i32 = i32(floor(sqrt(arrayLength(&kernel))));
    let half_kernel_size: i32 = kernel_size / 2;

    let src_size: vec2i = vec2i(textureDimensions(src));

    var sum: f32 = 0.;
    for (var y = -half_kernel_size; y <= half_kernel_size; y++) {
        let coord = igid + vec2i(0., y);

        if (coord.y < 0 || coord.y >= src_size.y) {
            return;
        }

        sum += kernel[y + half_kernel_size] * textureLoad(src, coord, 0);
    }

    sum = clamp(0., 1., sum);
    textureStore(dst, gid, vec4f(sum, 0., 0., 0.));
}