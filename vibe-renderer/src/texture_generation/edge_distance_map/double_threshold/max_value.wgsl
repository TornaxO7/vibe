@group(0) @binding(0)
var<storage, read_write> max_value: atomic<u32>;

@group(0) @binding(1)
var src: texture_storage_2d<r32float, read>;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let value = textureLoad(src, gid.xy).r;
    atomicMax(&max_value, bitcast<u32>(value));
}
