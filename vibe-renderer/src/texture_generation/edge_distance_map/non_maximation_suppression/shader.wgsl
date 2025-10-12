@group(0) @binding(0)
var src_infos: texture_storage_2d<rg32float, read>;

@group(0) @binding(1)
var dst: texture_storage_2d<r32float, write>;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    var value = 0.;
    if (!is_at_border(gid.xy)) {
        let igid = vec2i(gid.xy);
        let info = textureLoad(src_infos, gid.xy).rg;

        let magnitude = info.r;
        let radian = info.g;

        let dir = vec2f(cos(radian), sin(radian));
        let p1 = vec2f(igid) + dir;
        let p2 = vec2f(igid) - dir;

        let m1 = compute_magnitude(p1);
        let m2 = compute_magnitude(p2);

        if (m1 < magnitude && m2 < magnitude) {
            value = magnitude;
        }
    }

    textureStore(dst, gid.xy, vec4f(value, 0., 0., 0.));
}

fn is_at_border(coord: vec2u) -> bool {
    let size: vec2u = textureDimensions(dst);

    let x_is_at_border = coord.x == 0 || coord.x == size.x - 1;
    let y_is_at_border = coord.y == 0 || coord.y == size.y - 1;

    return x_is_at_border || y_is_at_border;
}

fn compute_magnitude(coord: vec2f) -> f32 {
    let id = vec2u(floor(coord));
    let sgv = smoothstep(vec2f(0.), vec2f(1.), fract(coord));

    let tl = id + vec2u(0, 0);
    let tr = id + vec2u(1, 0);
    let bl = id + vec2u(0, 1);
    let br = id + vec2u(1, 1);

    let mtl = textureLoad(src_infos, tl).r;
    let mtr = textureLoad(src_infos, tr).r;
    let mbl = textureLoad(src_infos, bl).r;
    let mbr = textureLoad(src_infos, br).r;

    let m1 = mix(mtl, mtr, sgv.x);
    let m2 = mix(mbl, mbr, sgv.x);
    return mix(m1, m2, sgv.y);
}
