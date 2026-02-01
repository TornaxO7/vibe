// Holds the screen resolution.
//   - `iResolution[0]`: Width
//   - `iResolution[1]`: Height
@group(0) @binding(0)
var<uniform> iResolution: vec2f;

// Contains the presence of the playing audio.
// You can imagine this to be the height-value for the bar-shader.
//
// Note: You can get the length of the array with the `arrayLength` function: https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl-function-reference.html#func-arrayLength
@group(0) @binding(1)
var<storage, read> freqs: array<f32>;

// Contains the time how long the shader has been running.
@group(0) @binding(2)
var<uniform> iTime: f32;

// Contains the (x, y) coordinate of the mouse.
// `x` and `y` are within the range [0, 1]:
//   - (0, 0) => top left corner
//   - (1, 0) => top right corner
//   - (0, 1) => bottom left corner
//   - (1, 1) => bottom right corner
@group(0) @binding(3)
var<uniform> iMouse: vec2f;

// Contains the detected BPM (beats per minute) of the audio.
// Typically in the range 60-200 for most music.
// Use this to sync animations to the music tempo.
@group(0) @binding(4)
var<uniform> iBPM: f32;

// Color palette for shader customization.
// Each color is a vec4 where xyz = RGB (0.0-1.0), w = 1.0.
// Configure via ~/.config/vibe/colors.toml
struct ColorPalette {
    color1: vec4f,
    color2: vec4f,
    color3: vec4f,
    color4: vec4f,
}

@group(0) @binding(5)
var<uniform> iColors: ColorPalette;
