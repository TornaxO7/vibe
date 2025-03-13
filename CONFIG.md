# General config file

`~/.config/vibe-daemon/config.toml` contains the config `vibe-daemon` itself and has the following options:

```toml
[graphics_config]
# Decide which gpu vibe should prefer.
# Can be either "low-power" (often your integrated GPU) or "high-performance" (your external GPU)
power_preference = "low-power"

# Set backend which you'd like to use. Can be any of those entries with `pub const <NAME>`: https://docs.rs/wgpu/latest/wgpu/struct.Backends.html#implementations
# Note:
#  - It's recommended to let it be `VULKAN`
#  - Writing each letter CAPITALIZED is required!
backend = "VULKAN"
```

## Output config file format

If you'd like to tweak around:
The config for each output can be seen in `~/.config/vibe-daemon/output_configs/<output-name>.toml`.

There you can add/remove/edit shaders of the given output. You can also add/use prewritten shaders from [vibe-shaders](https://github.com/TornaxO7/vibe-shaders/). See below for an example.

The config file has the following format:

```toml
# set to `false` if you don't want to have any shaders on the given output
enable = true

[[shaders]]

# configure the audio buffer for the shader. Currently you can only set the amount of bars
# you'd like to get in the shader
[shader.audio]
amount_bars = 60

# now you can add any amount of shader code you'd like to render.
# You can add any amount of `[[shader_code]]` which will be rendered.
# This one gets rendered first
[shader.code]
# choose which shader language will be used. Can be either `Glsl` or `Wgsl`.
# This just renders the screen red
Glsl = """
layout(set = 0, binding = 0) uniform float iTime;

// x: width
// y: height
layout(set = 1, binding = 0) uniform vec2 iResolution;

// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.
layout(set = 2, binding = 0) readonly buffer iAudio {
    float[] freqs;
};

// the color which the pixel should have
layout(location = 0) out vec4 fragColor;

void main() {
   fragColor = vec4(1., 0., 0., 1.);
}
"""

[[shaders]]

[shader.audio]
amount_bars = 60

[shader.code]
# You can also pick one of the pre-conifgured shaders: https://github.com/TornaxO7/vibe-shaders/
# Just enter the directory name here.
VibeShader = "galaxy_pulse"

# this will be rendered next
[[shaders]]

[shader.audio]
amount_bars = 60

[shader.code]
Wgsl = """
@group(0) @binding(0)
var<uniform> iTime: f32;

// x: width
// y: height
@group(1) @binding(0)
var<uniform> iResolution: vec2<f32>;

// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.
@group(2) @binding(0)
var<storage, read> iAudio: array<f32>;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(0.);
}"""
```

If you want to have a live-renderer to tweak around easier, just compile [shady-toy] with the following command: `cargo run --release --no-default-features --features=audio,resolution,time`.
`vibe` uses the same library as [shady-toy] and hence it should produce compatible shaders for `vibe`.
