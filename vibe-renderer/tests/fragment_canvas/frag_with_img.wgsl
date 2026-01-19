@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / iResolution.xy;

    return textureSample(iTexture, iSampler, uv);
}
