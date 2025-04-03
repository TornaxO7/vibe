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

### Location

Each output (monitor) has its own config file which can you find in `~/.config/vibe-daemon/output_configs/<output-name>.toml`.

### Format

#### Global format

It has the following structure:

```toml
# set to `false` if you don't want to have any shaders on the given output
enable = true

[[components]]
# now add as many components/shaders as you want. They will be rendered sequentially.
# See below for a list of components which you can copy+paste and tweak afterwards
```

## Components

Here's a list of components which you can use.
You can simply copy+paste the code blocks to add them to your config.

### Bars

Displays the frequency bars as in cava.

```toml
[components.Bars]
# set the maximum height for the bars.
# - `1.0` means 100% (it will use your ful monitor height for the bars)
# - `0.25` means 25% of your full monitor height will be used for the bars
max_height = 0.75

# set the amount of bars you'd like to display
[components.Bars.audio_conf]
amount_bars = 60

# Set the frequency range (in Hz) which it should visualize.
[components.Bars.audio_conf.freq_range]
start = 50
end = 10000

# Set the bar sensitivity.
# - `min`: The minimal changing rate for each bar. Especially interesting for very small changes.
#          If the bars are "jittering" too much in your opinion then you can decrease this value otherwise increase the value.
# - `max`: The maximal changing rate for each bar. Especially interesting for big changes (jump a high distance down or up).
#          If you want the bars to jump "slower", you can decrease this value, otherwise increase the value.
[components.Bars.audio_conf.sensitivity]
min = 0.05000000074505806
max = 0.20000000298023224

# The fragment code which should be used to color the bars.
#
# There are some global "variales" which you can use in the fragment shader:
# - For `wgsl`: https://github.com/TornaxO7/vibe/blob/main/vibe-renderer/src/components/bars/fragment_preamble.wgsl
# - For `glsl`: https://github.com/TornaxO7/vibe/blob/main/vibe-renderer/src/components/bars/fragment_preamble.glsl
[components.Bars.fragment_code]
# EITHER write WGSL code
Wgsl = """
@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var color = sin(vec3<f32>(2., 4., 8.) * iTime * .25) * .2 + .6;

    // apply gamma correction
    const GAMMA: f32 = 2.2;
    color.r = pow(color.r, GAMMA);
    color.g = pow(color.g, GAMMA);
    color.b = pow(color.b, GAMMA);
    return vec4<f32>(color, 1. - pos.y / iResolution.y);
}
"""
# OR GLSL code (uncomment after the "==") if you want to use it and remove the `Wsgl = ` part).
# ==
# Glsl = """
# void main() {
#     vec3 col = sin(vec3(2., 4., 8.) * iTime * .25) * .2 + .6;
#
#     const float GAMMA = 2.2;
#     col.r = pow(col.r, GAMMA);
#     col.g = pow(col.g, GAMMA);
#     col.b = pow(col.b, GAMMA);
#     fragColor = vec4(col, 1. - gl_FragCoord.y / iResolution.y);
# }
# """
```

# Extra information

If you want to have a live-renderer to tweak around easier, just compile [shady-toy] with the following command: `cargo run --release --no-default-features --features=audio,resolution,time`.
`vibe` uses the same library as [shady-toy] and hence it should produce compatible shaders for `vibe`.

### Fragment canvas

A
