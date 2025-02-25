# Vibe

A desktop [glava] like desktop music visualizer by using shaders!

# Demo

[![Demo video](https://img.youtube.com/vi/557iYiWnXn0/maxresdefault.jpg)](https://www.youtube.com/watch?v=557iYiWnXn0)

# State

Stable. Just start the binary and "it should work".
Feel free to create an issue if you encounter a bug or if you have a feature request!

# Usage

1. Install `pavucontrol` (or any other tool to see which programs are currently recording and change the source).
2. You will need to install some development library headers:

```
sudo apt install librust-wayland-client-dev librust-alsa-sys-dev libxkbcommon-dev
```

3. Compile and run the binary with `cargo run --release`
4. Your microphone is very likely be catched as the audio source.
   To fix that open up `pavucontrol` and set the audio source (see: https://github.com/TornaxO7/shady?tab=readme-ov-file#shady-audio-doesnt-listen-to-my-systems-audio)

# Output config file format

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

[shady-toy]: https://github.com/TornaxO7/shady/tree/main/shady-toy
[glava]: https://github.com/jarcode-foss/glava
