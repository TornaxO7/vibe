@group(0) @binding(0)
var<uniform> iResolution: vec2<f32>;

@group(1) @binding(0)
var<uniform> iTime: f32;

@group(1) @binding(1)
var<storage, read> freqs: array<f32>;

