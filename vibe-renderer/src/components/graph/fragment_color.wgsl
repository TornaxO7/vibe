@group(0) @binding(0)
var<uniform> iResolution: vec2f;

@group(0) @binding(1)
var<uniform> max_height: f32;

@group(0) @binding(2)
var<uniform> color: vec4f;

@group(0) @binding(7)
var<uniform> smoothness: f32;

@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

// It treats as if the bars should go from bottom to top.
// The first argument is the position from this perspective
// and the second argument the canvas height where the bars go to.
fn get_presence(pos: vec2f, canvas_height: f32) -> f32 {
    let bar_height = freqs[u32(pos.x)];
    let relative_offset = pos.y / canvas_height;
    return smoothstep(relative_offset - smoothness, relative_offset, bar_height * max_height);
}

@fragment
fn bottom(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    // flip `pos.y`
    let new_pos = vec2f(pos.x, iResolution.y - pos.y);
    return color * get_presence(new_pos, iResolution.y);
}

@fragment
fn top(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    return color * get_presence(pos.xy, iResolution.y);
}

@fragment
fn right(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    // flip `pos.x`
    let new_pos = vec2f(pos.y, iResolution.x - pos.x);
    return color * get_presence(new_pos, iResolution.x);
}

@fragment
fn left(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    return color * get_presence(pos.yx, iResolution.x);
}
