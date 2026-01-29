@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let col = pos.xy / iResolution.xy + iTime + freqs[3];
    return vec4<f32>(col, 1., 1.);
}
