@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (2. * pos.xy - iResolution.xy) / iResolution.y;

    let r = length(uv);

    return vec4<f32>(r, freqs[3], sin(iTime), 1.);
}
