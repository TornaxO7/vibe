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

The config file has the following format:

```toml
# set to `false` if you don't want to have any shaders on the given output
enable = true
# set the amount of values which are going to be in `iAudio`.
# The lower frquencies are in the lower indexes (0, 1, 2, etc.) and go up the bigger the index is.
amount_bars = 60

# now you can add any amount of shader code you'd like to render.
# You can add any amount of `[[shader_code]]` which will be rendered.
# This one gets rendered first
[[shader_code]]
# choose which shader language will be used. Can be either `Glsl` or `Wgsl`.
# This just renders the screen red
Glsl = """
// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.
layout(binding = 0) readonly buffer iAudio {
    float[] freqs;
};

// x: width
// y: height
layout(binding = 1) uniform vec2 iResolution;

layout(binding = 2) uniform float iTime;

// the color which the pixel should have
layout(location = 0) out vec4 fragColor;

void main() {
   fragColor = vec4(1., 0., 0., 1.);
}
"""

[[shader_code]]
# You can also pick one of the pre-conifgured shaders: https://github.com/TornaxO7/vibe-shaders/
# Just enter the directory name here.
VibeShader = "galaxy_pulse"

# this will be rendered next
[[shader_code]]
Wgsl = """
// It contains the 'presence' of a frequency. The lower the index the lower is its frequency and the other way round.
// So for example, if you are interested in the bass, choose the lower indices.
@group(0) @binding(0)
var<storage, read> iAudio: array<f32, 20>;

// x: width
// y: height
@group(0) @binding(1)
var<uniform> iResolution: vec2<f32>;

@group(0) @binding(2)
var<uniform> iTime: f32;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
   return vec4<f32>(1., 0., 0., 1.);
}
"""
```

If you want to have a live-renderer to tweak around easier, just compile [shady-toy] with the following command: `cargo run --release --no-default-features --features=audio,resolution,time`.
`vibe` uses the same library as [shady-toy] and hence it should produce compatible shaders for `vibe`.
