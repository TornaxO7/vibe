struct Data {
    resolution: vec2f,
    position_offset: vec2f,
    color: vec4f,
    rotation: mat2x2f,
    radius: f32,
    spike_sensitivity: f32,
    freq_radiant_step: f32,
}

@group(0) @binding(0)
var<uniform> data: Data;

// @group(0) @binding(0)
// var<uniform> iResolution: vec2f;

// @group(0) @binding(1)
// var<uniform> radius: f32;

// @group(0) @binding(2)
// var<uniform> rotation: mat2x2f;

// @group(0) @binding(3)
// var<uniform> spike_sensitivity: f32;

// @group(0) @binding(4)
// var<uniform> freq_radiant_step: f32;

// @group(0) @binding(5)
// var<uniform> color: vec4f;

// @group(0) @binding(6)
// var<uniform> position_offset: vec2f;

@group(0) @binding(1)
var<storage, read> freqs: array<f32>;

const PI: f32 = 3.1415926535;

fn rotate(r: f32) -> mat2x2f {
    return mat2x2f(cos(r), -sin(r), sin(r), cos(r));
}

fn get_uv(pos: vec2f) -> vec2f {
    var uv = pos.xy / data.resolution.xy - data.position_offset;
    uv.x *= data.resolution.x / data.resolution.y;
    return data.rotation * uv;
}

@fragment
fn main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv: vec2f = get_uv(pos.xy);

    let freqs_length: u32 = arrayLength(&freqs);
    let radiant: f32 = abs(atan2(uv.y, uv.x));
    
    // Calculate the position of the uv. `floor`ing it, will return the index
    // of the lower frequency spike.
    let uv_freq_pos: f32 = radiant / data.freq_radiant_step;
    let prev_freq_idx: u32 = u32(floor(uv_freq_pos));
    let next_freq_idx: u32 = u32(ceil(uv_freq_pos));
    let normalized_uv_freq_pos: f32 = smoothstep(0., 1., fract(uv_freq_pos));

    // calculate the offset to the ring
    var radius_offset: f32 = 0.;
    if (prev_freq_idx == 0) {
        // before the first frequency spike

        radius_offset = mix(freqs[0] * .75, freqs[0], normalized_uv_freq_pos);

    } else if (prev_freq_idx == freqs_length) {
        // after the last frequency spike

        let last_freq = freqs[freqs_length - 1];
        radius_offset = mix(last_freq, last_freq * .75, normalized_uv_freq_pos);

    } else {

        // to be honest, I don't understand why I need to do `-1` here but somehow
        // those indexes are one off
        let prev_freq: f32 = freqs[prev_freq_idx - 1];
        let next_freq: f32 = freqs[next_freq_idx - 1];

        radius_offset = mix(prev_freq, next_freq, normalized_uv_freq_pos);

    }

    let x = abs(length(uv) - (data.radius + radius_offset * data.spike_sensitivity));
    let y = smoothstep(radius_offset * .01 + .005, .0, abs(x));
    return data.color * y;
}
