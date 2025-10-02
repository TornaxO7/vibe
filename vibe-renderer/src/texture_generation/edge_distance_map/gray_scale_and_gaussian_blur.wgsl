@group(0) @binding(0)
var src: texture_2d<f32>;

@group(0) @binding(1)
var dst: texture_storage_2d<r16unorm, write>;

@group(0) @binding(2)
var<storage, read> kernel: array<f32>;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let igid = vec2i(gid.xy);

    let kernel_size: i32 = i32(round(sqrt(f32(arrayLength(&kernel)))));
    let half_kernel_size: i32 = kernel_size / 2;
    let input_size: vec2i = vec2i(textureDimensions(src));

    var sum: f32 = 0.;
    for (var x = -half_kernel_size; x <= half_kernel_size; x++) {
        for (var y = -half_kernel_size; y <= half_kernel_size; y++) {
            let coords: vec2i = igid + vec2i(x, y);
            
            let is_over_the_top = coords.y < 0;
            let is_over_left_edge = coords.x < 0;
            let is_over_right_edge = coords.x >= input_size.x;
            let is_below_bottom_edge = coords.y >= input_size.y;
            if (is_over_the_top || is_over_left_edge || is_over_right_edge || is_below_bottom_edge) {
                return;
            }

            let texel = textureLoad(src, gid.xy, 0);
            let luminance = 0.3 * texel.r + 0.59 * texel.g + 0.11 * texel.b;

            let kernel_value = kernel[(x + half_kernel_size) + kernel_size * (y + half_kernel_size)];

            sum += kernel_value * luminance;
        }
    }

    sum = clamp(0., 1., sum);
    textureStore(output, gid.xy, vec4f(sum, 0., 0., 0.));
}
