@group(0) @binding(0)
var<uniform> iResolution: vec2f;

@group(0) @binding(1)
var<uniform> offset: vec2f;

@group(0) @binding(2)
var<uniform> rotation: mat2x2f;

@group(0) @binding(3)
var<uniform> max_height: f32;

@group(0) @binding(4)
var<uniform> color1: vec4f;

@group(0) @binding(5)
var<uniform> color2: vec4f;

@group(1) @binding(0)
var<storage, read> freqs: array<f32>;

fn get_freq(uv: vec2f) -> f32 {
    let amount_bars = arrayLength(&freqs);
    
    let freq_idx_i32 = i32(floor(uv.x * iResolution.x));
    // discard bars on the left and on the right
    if (freq_idx_i32 < 0 || u32(freq_idx_i32) > amount_bars) {
        discard;
    }

    let freq_idx = u32(freq_idx_i32);
    return freqs[freq_idx];
}

// Returns the uv coordinates as if it would always put the bars at the top.
fn get_normalized_uv(p: vec4f) -> vec2f {
    // normalize to [0, 1]x[0, 1]
    let uv = p.xy / iResolution.xy;

    // move to origin
    let origin_uv = uv - offset;

    // "reverse" the rotation
    return rotation * origin_uv;
}

// Returns the mask value for the given normalized uv coordinate.
fn get_mask(uv: vec2f) -> f32 {
    let freq = get_freq(uv);
    if (freq < 1e-5) {
        return 0.;
    } else {
        let height = freq * max_height;
        let bar_mask = smoothstep(height, height * .9, uv.y);
        let ground_mask = smoothstep(-.05, .0, uv.y);
        return bar_mask * ground_mask;
    }
}

@fragment
fn color(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let uv = get_normalized_uv(p);
    let mask = get_mask(uv);

    return color1 * mask;
}

@fragment
fn horizontal_gradient(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let uv = get_normalized_uv(p);
    let mask = get_mask(uv);

    let col = mix(color1, color2, uv.x);
    return col * mask;
}

@fragment
fn vertical_gradient(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let uv = get_normalized_uv(p);
    let mask = get_mask(uv);

    let col = mix(color1, color2, smoothstep(0., max_height, uv.y));
    return col * mask;
}

