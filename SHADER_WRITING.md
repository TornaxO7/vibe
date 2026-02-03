---
name: shader-writing
description: Use this skill when writing, debugging, or optimizing WGSL fragment shaders for vibe. Covers the full shader interface (uniforms, audio data, colors), visual techniques (raymarching, fractals, volumetrics, 4D geometry), performance optimization, and color palette integration.
---

# Writing WGSL Shaders for Vibe

Vibe is a Wayland audio visualizer that renders custom WGSL fragment shaders with real-time audio data. Every shader receives screen coordinates, frequency data, elapsed time, mouse position, BPM, and a 4-color palette. This guide covers everything needed to write shaders that look great, respond to music, and run efficiently.

## The Shader Interface

Every shader gets a **preamble** injected before your code. You do NOT declare these bindings yourself — they already exist when your `main()` runs.

### Available Bindings

| Binding | Type | Name | Description |
|---------|------|------|-------------|
| `@group(0) @binding(0)` | `vec2f` (uniform) | `iResolution` | Screen width and height in pixels |
| `@group(0) @binding(1)` | `array<f32>` (storage) | `freqs` | Audio frequency magnitudes (dynamic length) |
| `@group(0) @binding(2)` | `f32` (uniform) | `iTime` | Seconds since shader started |
| `@group(0) @binding(3)` | `vec2f` (uniform) | `iMouse` | Mouse position normalized to [0,1] |
| `@group(0) @binding(4)` | `f32` (uniform) | `iBPM` | Detected beats per minute (typically 60-200) |
| `@group(0) @binding(5)` | `ColorPalette` (uniform) | `iColors` | 4-color palette from `colors.toml` |
| `@group(0) @binding(6)` | `sampler` | `iSampler` | Texture sampler (for optional image) |
| `@group(0) @binding(7)` | `texture_2d<f32>` | `iTexture` | Optional texture image |

### ColorPalette Struct

```wgsl
struct ColorPalette {
    color1: vec4f,  // xyz = RGB (0.0-1.0), w = 1.0
    color2: vec4f,
    color3: vec4f,
    color4: vec4f,
}
```

### Minimal Shader

```wgsl
@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (pos.xy - iResolution.xy * 0.5) / min(iResolution.x, iResolution.y);
    var color = vec3<f32>(0.0);
    // ... your effect here ...
    return vec4<f32>(color, 1.0);
}
```

## Screen Coordinates

**Always normalize coordinates consistently.** The standard pattern centers the origin and preserves aspect ratio:

```wgsl
let uv = (pos.xy - iResolution.xy * 0.5) / min(iResolution.x, iResolution.y);
```

This gives `uv` centered at (0,0) with the shorter axis spanning [-0.5, 0.5]. Use this for effects where geometry should look correct regardless of window shape.

For full-screen UV (0 to 1, useful for backgrounds and gradients):

```wgsl
let uv = pos.xy / iResolution.xy;
```

## Audio Data

The `freqs` array contains frequency bar magnitudes processed from the audio stream. Values are adaptive — the audio pipeline normalizes them toward a 0.0-1.0 range over time, but spikes above 1.0 are common.

### Extracting Frequency Bands

```wgsl
let n_freqs = arrayLength(&freqs);

// Bass: lowest 4 bars
let bass = (freqs[0] + freqs[1] + freqs[2] + freqs[3]) / 4.0;

// Mid: center of the spectrum
let mid_idx = n_freqs / 2u;
let mid = (freqs[mid_idx] + freqs[min(mid_idx + 1u, n_freqs - 1u)]) / 2.0;

// Treble: highest bars
let high_idx = n_freqs - 2u;
let treble = (freqs[high_idx] + freqs[min(high_idx + 1u, n_freqs - 1u)]) / 2.0;
```

Always use `min(..., n_freqs - 1u)` as a bounds guard when indexing adjacent bars.

### Audio Reactivity Patterns

Different scaling functions give different visual character:

| Pattern | Code | Feel |
|---------|------|------|
| **Linear** | `bass * 1.5` | Direct, punchy |
| **Logarithmic** | `log(1.0 + mid * 6.0)` | Compressed, musical |
| **Square root** | `sqrt(treble + 0.01)` | Gentle, responsive to quiet |
| **Exponential** | `exp(treble * 2.0) - 1.0` | Explosive on peaks |
| **Sine modulation** | `sin(iTime + mid * 6.0)` | Rhythmic, oscillating |
| **Amplitude envelope** | `1.0 + bass * 0.5` | Subtle brightness multiplier |

### How Much Audio Reactivity

The audio should enhance the visual, not dominate it. Guidelines:

- **Background wallpaper shaders**: Very subtle. Multiply final color by `0.9 + bass * 0.15`. Use audio for slow drift, not pulsing. See `plasma.wgsl` — audio warmth is `bass * 0.08 + mid * 0.05`.
- **Window visualizers**: Moderate. Audio drives structure — ring radii, fractal zoom offsets, particle velocities.
- **Intense visualizers**: Full reactivity. Bass shakes cameras, treble drives glow intensity, mid modulates geometry.

The key rule: **the shader should still look good with no audio playing.** Audio adds life, but silence shouldn't mean a blank screen.

### Mapping Frequency to Angle (Radial Visualizers)

```wgsl
let normalized_angle = (angle + PI) / (2.0 * PI);
let freq_idx = u32(normalized_angle * f32(arrayLength(&freqs)));
let freq = freqs[min(freq_idx, arrayLength(&freqs) - 1u)];
```

## Color Palette

Colors come from `~/.config/vibe/colors.toml`:

```toml
color1 = [0.08, 0.10, 0.18]   # Dark background
color2 = [0.15, 0.20, 0.35]   # Secondary/warm
color3 = [0.25, 0.35, 0.50]   # Primary/body
color4 = [0.30, 0.25, 0.45]   # Accent/glow
```

### Color Roles

Assign semantic meaning to each color slot. The convention used across existing shaders:

| Slot | Role | Usage |
|------|------|-------|
| `color1` | **Background / darkest** | Deep space, interior of fractals, fog target |
| `color2` | **Secondary / warm** | Warm highlights, outer regions, starfield tint |
| `color3` | **Primary / body** | Main geometry surface, mid-tones |
| `color4` | **Accent / glow** | Brightest elements, emissive edges, hot regions |

### Accessing Colors

```wgsl
let c1 = iColors.color1.xyz;  // Background
let c2 = iColors.color2.xyz;  // Secondary
let c3 = iColors.color3.xyz;  // Primary
let c4 = iColors.color4.xyz;  // Accent
```

### Blending Across the Palette

**Linear interpolation between two colors:**
```wgsl
let color = mix(c1, c2, t);  // t in [0, 1]
```

**Cycling through all four colors with a parameter `t` in [0, 1]:**
```wgsl
var color: vec3<f32>;
if (t < 0.33) {
    color = mix(c1, c2, t * 3.0);
} else if (t < 0.66) {
    color = mix(c2, c3, (t - 0.33) * 3.0);
} else {
    color = mix(c3, c4, (t - 0.66) * 3.0);
}
```

**Smooth transitions with depth or distance:**
```wgsl
// Near = accent, far = background
let col = mix(c4, c1, smoothstep(0.0, 1.0, depth));
```

### Making Colors Work Well

- **Never hardcode RGB values.** Always derive from the palette. This ensures the shader looks good with any user-configured color scheme.
- **Use `mix()` between palette colors**, not between palette and hardcoded. Exception: `vec3<f32>(1.0)` for specular highlights is acceptable.
- **Fresnel effects** use the palette: `mix(col_body, col_glow, fresnel * 0.65)`.
- **Emissive glow** comes from the accent: `col_glow * (0.5 + bass * 0.9)`.
- **Fog** fades toward the background: `mix(color, col_bg * 0.03, fog)`.
- **Starfield or ambient tint** uses secondary: `star_color = vec3<f32>(0.6) + col_warm * 0.4`.

## Shader Structure Template

```wgsl
const BRIGHTNESS: f32 = 1.4;
const PI: f32 = 3.14159265;

// Helper functions here

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (pos.xy - iResolution.xy * 0.5) / min(iResolution.x, iResolution.y);

    // 1. Extract audio
    let n_freqs = arrayLength(&freqs);
    let bass = (freqs[0] + freqs[1] + freqs[2] + freqs[3]) / 4.0;
    let mid_idx = n_freqs / 2u;
    let mid = (freqs[mid_idx] + freqs[min(mid_idx + 1u, n_freqs - 1u)]) / 2.0;
    let high_idx = n_freqs - 2u;
    let treble = (freqs[high_idx] + freqs[min(high_idx + 1u, n_freqs - 1u)]) / 2.0;

    // 2. Get palette
    let c1 = iColors.color1.xyz;
    let c2 = iColors.color2.xyz;
    let c3 = iColors.color3.xyz;
    let c4 = iColors.color4.xyz;

    // 3. Compute your effect
    var color = vec3<f32>(0.0);
    // ...

    // 4. Post-processing
    // Tone mapping (Reinhard) — prevents blown-out highlights
    color = color / (color + vec3<f32>(1.0));
    // Vignette — darkens edges naturally
    color *= 1.0 - dot(uv, uv) * 0.3;

    return vec4<f32>(color * BRIGHTNESS, 1.0);
}
```

## Essential Building Blocks

### Procedural Hash (Pseudo-Random)

```wgsl
fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}
```

### Gradient Noise

```wgsl
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);  // Smoothstep interpolation
    return mix(
        mix(hash21(i), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}
```

### Fractional Brownian Motion (FBM)

Layered noise at different scales. Essential for organic textures — clouds, nebulae, terrain.

```wgsl
fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var pp = p;
    for (var i = 0; i < octaves; i++) {
        v += a * noise(pp);
        pp = pp * 2.0;
        a *= 0.5;
    }
    return v;
}
```

**Performance note**: Each octave doubles the cost. Use 4-5 octaves for backgrounds, 3 for real-time effects.

### 3D Noise (for Volumetrics)

```wgsl
fn noise3d(p: vec3<f32>) -> f32 {
    let n1 = noise(p.xy + p.z * 17.0);
    let n2 = noise(p.yz + p.x * 13.0);
    return (n1 + n2) * 0.5;
}
```

### Smooth Minimum (Soft Blending)

Blends distance fields smoothly instead of hard `min()`. Essential for organic shapes.

```wgsl
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}
```

`k` controls blend radius: 0.1 = subtle merge, 0.5 = blobby merge.

### Distance to Line Segment

```wgsl
fn dist_seg(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let t = clamp(dot(pa, ba) / (dot(ba, ba) + 0.0001), 0.0, 1.0);
    return length(pa - ba * t);
}
```

### Rotation Matrices

```wgsl
fn rotY(a: f32) -> mat3x3<f32> {
    let c = cos(a); let s = sin(a);
    return mat3x3<f32>(
        vec3<f32>(c, 0.0, -s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(s, 0.0, c)
    );
}

fn rotX(a: f32) -> mat3x3<f32> {
    let c = cos(a); let s = sin(a);
    return mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, c, s),
        vec3<f32>(0.0, -s, c)
    );
}
```

Apply: `let p_rotated = rotY(angle) * p;`

## Technique: Raymarching (3D Scenes)

Raymarching renders 3D scenes by stepping along rays and testing distance to geometry using Signed Distance Functions (SDFs).

### SDF Primitives

```wgsl
fn sdSphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sdBox(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sdRoundBox(p: vec3<f32>, b: vec3<f32>, r: f32) -> f32 {
    return sdBox(p, b) - r;
}

fn sdOctahedron(p: vec3<f32>, s: f32) -> f32 {
    let q = abs(p);
    return (q.x + q.y + q.z - s) * 0.57735027;
}

fn sdTorus(p: vec3<f32>, major: f32, minor: f32) -> f32 {
    let q = vec2<f32>(length(p.xz) - major, p.y);
    return length(q) - minor;
}
```

### Combining SDFs

```wgsl
min(a, b)           // Union (hard)
max(a, b)           // Intersection
max(-a, b)          // Subtraction (cut a from b)
smin(a, b, k)       // Smooth union (organic merge)
mix(a, b, t)        // Morphing between shapes
```

### The Raymarch Loop

```wgsl
const MAX_STEPS: i32 = 80;
const MAX_DIST: f32 = 40.0;
const SURF_DIST: f32 = 0.002;

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> vec2<f32> {
    var t = 0.0;
    var mat_id = -1.0;
    for (var i = 0; i < MAX_STEPS; i++) {
        let h = scene(ro + rd * t);
        if h.x < SURF_DIST {
            mat_id = h.y;
            break;
        }
        t += h.x;
        if t > MAX_DIST { break; }
    }
    return vec2<f32>(t, mat_id);  // distance + material ID
}
```

Return material IDs (0.0, 1.0, 2.0...) from the scene function to color different objects differently.

### Normals (Central Differences)

```wgsl
fn getNormal(p: vec3<f32>) -> vec3<f32> {
    let e = 0.001;
    let d = scene(p).x;
    return normalize(vec3<f32>(
        scene(p + vec3<f32>(e, 0.0, 0.0)).x - d,
        scene(p + vec3<f32>(0.0, e, 0.0)).x - d,
        scene(p + vec3<f32>(0.0, 0.0, e)).x - d
    ));
}
```

### Camera Setup

```wgsl
let ro = vec3<f32>(cos(cam_angle) * 8.0, 3.5, sin(cam_angle) * 8.0);
let look_at = vec3<f32>(0.0, 2.0, 0.0);

let fwd = normalize(look_at - ro);
let right = normalize(cross(fwd, vec3<f32>(0.0, 1.0, 0.0)));
let up = cross(right, fwd);
let rd = normalize(fwd * 1.5 + right * uv.x + up * uv.y);
```

The focal length (1.5 here) controls field of view. Lower = wider.

### Lighting

**Diffuse + Specular + Fresnel:**
```wgsl
let diff = max(dot(n, light_dir), 0.0);
let half_v = normalize(light_dir - rd);
let spec = pow(max(dot(n, half_v), 0.0), 64.0);
let fresnel = pow(1.0 - max(dot(n, -rd), 0.0), 3.0);
```

**Ambient Occlusion** (cheap approximation — 5 SDF queries):
```wgsl
fn calcAO(p: vec3<f32>, n: vec3<f32>) -> f32 {
    var occ = 0.0;
    var w = 1.0;
    for (var i = 0; i < 5; i++) {
        let h = 0.01 + 0.12 * f32(i);
        occ += (h - scene(p + n * h).x) * w;
        w *= 0.85;
    }
    return clamp(1.0 - 3.0 * occ, 0.0, 1.0);
}
```

**Soft Shadows** (24 steps along light ray):
```wgsl
fn softShadow(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var res = 1.0;
    var t = 0.05;
    for (var i = 0; i < 24; i++) {
        let h = scene(ro + rd * t).x;
        res = min(res, 12.0 * h / t);
        t += clamp(h, 0.02, 0.25);
        if res < 0.001 || t > 10.0 { break; }
    }
    return clamp(res, 0.0, 1.0);
}
```

### Shape Morphing

Interpolate between SDF primitives for audio-reactive geometry:

```wgsl
let phase = fract(iTime * 0.08 + bass * 0.08) * 3.0;
let d_oct = sdOctahedron(p, 1.2 + bass * 0.15);
let d_box = sdRoundBox(p, vec3<f32>(0.85), 0.12);
let d_sph = sdSphere(p, 1.1);

var shape: f32;
if phase < 1.0 {
    shape = mix(d_oct, d_box, smoothstep(0.0, 1.0, phase));
} else if phase < 2.0 {
    shape = mix(d_box, d_sph, smoothstep(0.0, 1.0, phase - 1.0));
} else {
    shape = mix(d_sph, d_oct, smoothstep(0.0, 1.0, phase - 2.0));
}
```

### Floor Reflections

After the primary hit, cast a reflection ray:

```wgsl
let refl_rd = reflect(rd, n);
let refl = raymarch(p + n * 0.03, refl_rd);
if refl.y >= 0.0 {
    let rp = p + refl_rd * refl.x;
    let rn = getNormal(rp);
    // ... shade reflection, blend with floor color
}
```

### Volumetric God Rays

Sample along the ray in front of the camera, accumulate glow:

```wgsl
var vol = 0.0;
for (var i = 0; i < 16; i++) {
    let t = f32(i) * 0.6 + 0.5;
    if hit.y >= 0.0 && t > hit.x { break; }
    let sp = ro + rd * t;
    let to_axis = length(sp.xz);
    let cone = smoothstep(2.5, 0.0, to_axis) * smoothstep(8.0, 2.0, sp.y);
    vol += cone * 0.025;
}
color += col_glow * vol * (0.2 + bass * 0.3);
```

## Technique: Fractals

### Mandelbrot Set

The classic complex number iteration. The key to making it visually interesting is **smooth iteration count** and **multi-zone audio reactivity**.

```wgsl
// Early bailout for known interior regions (massive speedup)
fn in_cardioid(x: f32, y: f32) -> bool {
    let q = (x - 0.25) * (x - 0.25) + y * y;
    return q * (q + (x - 0.25)) <= 0.25 * y * y;
}

fn in_bulb(x: f32, y: f32) -> bool {
    let dx = x + 1.0;
    return dx * dx + y * y <= 0.0625;
}
```

**Smooth iteration count** eliminates banding:
```wgsl
var smooth_iter = f32(iter);
if (iter < max_iter - 1) {
    let log_zn = log(dot(z, z)) / 2.0;
    let nu = log(log_zn / log(2.0)) / log(2.0);
    smooth_iter = f32(iter) + 1.0 - nu;
}
```

**Multi-class contours** — different audio bands drive different depth zones:
```wgsl
let depth = smooth_iter / f32(max_iter);
var offset: f32 = 0.0;

// Outer region: linear bass response
offset += (1.0 - smoothstep(0.0, 0.25, depth)) * bass * 5.0;

// Mid region: logarithmic mid response
let mo = smoothstep(0.15, 0.25, depth) * (1.0 - smoothstep(0.35, 0.45, depth));
offset += mo * log(1.0 + mid * 6.0) * 2.0;

// Inner region: exponential treble response
offset += smoothstep(0.75, 0.85, depth) * (exp(treble * 2.0) - 1.0);

// Use offset to shift color cycling
let t = fract((smooth_iter + offset + iTime * 0.2) / 20.0);
```

This creates distinct visual layers that respond to different parts of the music.

### Fractal Performance

- Use `max_iter = 64`, not 256. 64 is enough for most zoom levels.
- **Cardioid and bulb bailout** skip ~50% of pixels in the Mandelbrot set interior — they're always black, so don't iterate them.
- Slow zoom: `let zoom = 1.5 / (1.0 + iTime * 0.05);`

## Technique: Volumetric Layers

Layer multiple 2D noise planes at different depths for a 3D volumetric feel without raymarching.

```wgsl
let num_layers = 8;
for (var layer = 0; layer < num_layers; layer++) {
    let depth = f32(layer) / f32(num_layers);

    // Parallax: deeper layers move slower
    let parallax = 1.0 - depth * 0.7;
    let layer_uv = uv_rot * parallax + vec2<f32>(f32(layer) * 3.7, f32(layer) * 2.3);

    let n = fbm(layer_uv * (2.0 + depth * 2.0) + t, 5);

    // Depth-based color: near = accent, far = background
    var layer_color = mix(c3, c1, depth);

    // Audio: bass pushes near layers, treble lights far layers
    let audio_boost = (1.0 - depth) * bass * 1.5 + depth * treble * 0.8;

    color += layer_color * n * (0.3 + audio_boost);
}
color /= f32(num_layers) * 0.4;
```

## Technique: 4D Geometry

Project higher-dimensional objects into 2D. The tesseract (4D hypercube) is the canonical example.

### Tesseract Vertex Generation

A tesseract has 16 vertices. Each is encoded as a 4-bit index where each bit maps to -1 or +1:

```wgsl
fn vert4d(idx: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(idx >> 3u & 1u) * 2.0 - 1.0,
        f32(idx >> 2u & 1u) * 2.0 - 1.0,
        f32(idx >> 1u & 1u) * 2.0 - 1.0,
        f32(idx & 1u) * 2.0 - 1.0
    );
}
```

### 4D Rotations

Each rotation operates in a 2D plane within 4D space:

```wgsl
fn rotXW(v: vec4<f32>, a: f32) -> vec4<f32> {
    let c = cos(a); let s = sin(a);
    return vec4<f32>(v.x * c - v.w * s, v.y, v.z, v.x * s + v.w * c);
}
```

These produce the characteristic "inside-out" morphing. Audio drives the rotation angles for hyperdimensional spin.

### 4D Perspective Projection

```wgsl
fn project4to3(v: vec4<f32>, w_cam: f32) -> vec3<f32> {
    let s = w_cam / (w_cam - v.w);
    return v.xyz * s;
}
```

`w_cam` controls the 4D "depth of field" — modulate with mid for subtle breathing.

### Edge Detection

Two tesseract vertices share an edge if they differ in exactly one bit:

```wgsl
for (var i = 0u; i < 16u; i++) {
    for (var bit = 0u; bit < 4u; bit++) {
        let j = i ^ (1u << bit);
        if j <= i { continue; }  // Avoid drawing each edge twice
        // ... draw edge between scr[i] and scr[j]
    }
}
```

Color W-bridge edges (bit 0) differently from XYZ edges for visual distinction.

## Technique: Gravitational Lensing

Trace photon paths through curved spacetime. Instead of straight rays, deflect them gravitationally each step:

```wgsl
let grav = 1.5 * RS * (1.0 + bass * 0.25);

for (var i = 0; i < RAY_STEPS; i++) {
    let r = length(ray_pos);

    if r < RS * 1.1 { absorbed = true; break; }  // Event horizon

    // Adaptive step: tiny near the singularity, large far away
    let step_size = max(0.08, (r - RS) * 0.25);

    // Gravitational deflection — bends the ray
    let accel = normalize(-ray_pos) * grav / (r * r);
    ray_dir = normalize(ray_dir + accel * step_size);

    ray_pos = ray_pos + ray_dir * step_size;

    if r > 60.0 { break; }  // Escaped
}
```

### Accretion Disk

Check for disk plane crossings during ray integration (y sign change). Use Keplerian velocity for spiral structure and Doppler beaming for brightness variation:

```wgsl
if ray_pos.y * next_pos.y < 0.0 {
    let t_cross = ray_pos.y / (ray_pos.y - next_pos.y);
    let cross_pos = ray_pos + ray_dir * step_size * t_cross;
    // Sample disk at cross_pos — spiral pattern, zone colors
}
```

## Technique: Soft Bodies (Lava Lamp / Plasma)

Multiple overlapping blobs with smooth distance field merging.

### Blob Distance Fields

```wgsl
fn blobShape(p: vec2<f32>, center: vec2<f32>, radius: f32, t: f32, seed: f32) -> f32 {
    let d = p - center;
    let angle = atan2(d.y, d.x);

    // Slow organic wobble
    var wobble = sin(angle * 2.0 + t * 0.08 + seed) * 0.3;
    wobble += sin(angle * 3.0 + t * 0.05 + seed * 1.7) * 0.2;

    return length(d) - radius - radius * 0.15 * wobble;
}
```

### Merge Many Blobs

```wgsl
var combined_dist = 1000.0;
for (var i = 0; i < NUM_BLOBS; i++) {
    let d = blobShape(p, center, radius, t, seed);
    combined_dist = smin(combined_dist, d, 0.2);  // Smooth merge

    // Weight-based color blending
    let weight = 1.0 / (1.0 + max(d, 0.0) * 6.0);
    blended_color += blob_color * weight;
    total_weight += weight;
}
let blob_mask = 1.0 - smoothstep(-0.015, 0.015, combined_dist);
```

### Convection

Simulate rising/falling with very long periods (60-120 seconds per cycle):

```wgsl
fn convectionY(phase: f32) -> f32 {
    let p = fract(phase);
    let y = sin(p * PI) * 0.5 + 0.5;
    return 0.08 + y * y * (3.0 - 2.0 * y) * 0.84;
}
```

## Technique: Star Fields

### Procedural Star Placement

```wgsl
fn star_layer(uv: vec2<f32>, scale: f32, time_offset: f32) -> vec3<f32> {
    let scaled = uv * scale;
    let cell = floor(scaled);
    let cell_frac = fract(scaled) - 0.5;

    let h = hash21(cell);
    if h < 0.96 { return vec3<f32>(0.0); }  // Most cells are empty

    let offset = vec2<f32>(hash21(cell * 1.3) - 0.5, hash21(cell * 1.7) - 0.5) * 0.6;
    let d = length(cell_frac - offset);
    let twinkle = 0.7 + 0.3 * sin(iTime * 1.5 + h * 80.0);
    let brightness = smoothstep(0.08, 0.0, d) * twinkle;

    return vec3<f32>(1.0) * brightness;
}
```

Use 3 layers at different scales with parallax for depth.

### Spherical Star Projection (for 3D scenes)

When you have a ray direction, project onto spherical coordinates for uniform sky distribution:

```wgsl
let theta = atan2(rd.z, rd.x);
let phi = asin(clamp(rd.y, -1.0, 1.0));
let sky = vec2<f32>(theta, phi);
// Use sky as UV for star placement
```

## Post-Processing

### Reinhard Tone Mapping

Prevents blown-out highlights while preserving color ratios:

```wgsl
color = color / (color + vec3<f32>(1.0));
```

Apply this **before** the final brightness multiplier. Essential for any shader that accumulates light (volumetrics, glow effects, emissive materials).

### Vignette

Natural edge darkening. Pick the intensity based on shader style:

```wgsl
// Subtle (backgrounds)
color *= 1.0 - dot(uv, uv) * 0.2;

// Medium (most shaders)
color *= 1.0 - dot(uv, uv) * 0.3;

// Strong (dramatic)
color *= 1.0 - length(uv) * length(uv) * 0.4;
```

### Distance Fog

For 3D scenes, fade to background color with distance:

```wgsl
let fog = 1.0 - exp(-hit.x * hit.x * 0.002);
color = mix(color, col_bg * 0.03, fog);
```

### Glow Effects

Inverse-distance glow for emissive elements:

```wgsl
let glow = thickness / (distance + thickness);  // Soft glow
let glow = exp(-distance * falloff);             // Exponential glow
```

## Performance

### Budget

Vibe shaders run every frame at monitor refresh rate. At 4K 144Hz, each fragment runs billions of times per second. Performance matters.

### Step Count Guidelines

| Element | Max Steps | Notes |
|---------|-----------|-------|
| Raymarch primary | 80 | Sufficient for most scenes |
| Raymarch reflection | 40-60 | Lower fidelity is acceptable |
| Soft shadows | 24 | Increase step size aggressively |
| Ambient occlusion | 5 | Cheap but effective |
| God rays / volumetrics | 16 | Sample sparsely |
| FBM octaves | 4-5 | Each octave doubles noise cost |
| Volumetric depth layers | 6-8 | Trade-off with FBM octaves |

### Optimization Techniques

1. **Early bailout**: Fractals: skip known interior (cardioid/bulb). Raymarching: `if t > MAX_DIST { break; }`.

2. **Adaptive step sizing**: Near complex geometry, use small steps. Far away, step aggressively.
   ```wgsl
   let step_size = max(0.08, (r - RS) * 0.25);
   t += clamp(h, 0.02, 0.25);
   ```

3. **Limit loop iterations**: 5 orbiting objects, not 20. 8 depth layers, not 16. The visual difference between 5 and 20 satellites is rarely worth 4x the cost.

4. **Pre-compute trig pairs**: If you need both `sin(a)` and `cos(a)`, compute once and reuse.

5. **Avoid unnecessary branching in hot loops**: Use `smoothstep()` for soft transitions instead of `if/else` where possible. GPUs run both branches of a conditional if any thread in a warp takes it.

6. **Use `smoothstep` over `clamp` for visual transitions**: `smoothstep` is the same cost as `clamp` on GPU but looks significantly better.

7. **Simple hash functions**: `fract(sin(dot(p, ...)) * 43758.5453)` is fast enough. Don't import complex PRNG.

8. **Tone map once at the end**: Don't clamp intermediate values — let HDR accumulate, then map to [0,1] with Reinhard at the final step.

### GPU Load Monitoring

```bash
nvidia-smi    # Check GPU utilization and temperature
```

Supersampling in vibe config multiplies GPU load proportionally. A shader that runs at 60% GPU at 1x will run at 240% (dropped frames) at 4x.

## Output Config Reference

Shaders are loaded via TOML config files in `~/.config/vibe/output_configs/`:

```toml
enable = true

[[components]]
[components.FragmentCanvas.audio_conf]
amount_bars = 128
sensitivity = 3.0
freq_range.Custom = { start = 20, end = 16000 }

[components.FragmentCanvas.fragment_code]
language = "Wgsl"
path = "/absolute/path/to/shader.wgsl"
```

### Config Fields

| Field | Type | Description |
|-------|------|-------------|
| `amount_bars` | `u16` | Number of frequency bins in `freqs` array (more = finer spectral resolution) |
| `sensitivity` | `f32` | Audio input multiplier (higher = more reactive) |
| `freq_range.Custom` | `{ start, end }` | Frequency range in Hz to capture |
| `language` | `"Wgsl"` or `"Glsl"` | Shader language |
| `path` | string | Absolute path to shader file |

### Preset Frequency Ranges

Instead of `freq_range.Custom`, you can use:
- `freq_range = "Bass"` — low frequencies only
- `freq_range = "Mid"` — mid frequencies only
- `freq_range = "Treble"` — high frequencies only

### Optional Texture

```toml
[components.FragmentCanvas.texture]
path = "/absolute/path/to/image.png"
```

Access in shader: `let col = textureSample(iTexture, iSampler, uv).rgb;`

### Hot Reload

In window mode (`vibe window-1`), config and shader files are watched with inotify. Save the file and the shader reloads automatically — no restart needed. The audio pipeline's normalize_factor starts conservatively low and ramps up, so reloaded shaders fade in smoothly.

## Avoiding Dull Colors

The fastest way to kill a shader's visual appeal is to average colors together. Averaging converges toward gray — every `mix()` with `t = 0.5` and every `/4.0` across the palette pulls the output toward muddy middle tones. The palette has four distinct colors for a reason. Keep them distinct.

### The Averaging Trap

```wgsl
// BAD: averaging the entire palette produces gray mud
let color = (c1 + c2 + c3 + c4) / 4.0;

// BAD: 50/50 blend of complementary colors → desaturated
let color = mix(c1, c4, 0.5);

// BAD: blending all audio bands into one signal, then blending all colors
let energy = (bass + mid + treble) / 3.0;
let color = mix(mix(c1, c2, energy), mix(c3, c4, energy), 0.5);
```

Every blend operation loses saturation. Chain two blends and the color is already noticeably duller than either input.

### Keep Colors Separated by Role

Each color should dominate in its own region. Don't blend them all into the same pixel.

```wgsl
// GOOD: each color owns a spatial zone
if depth < 0.3 {
    color = c4 * 1.3;              // Hot inner — accent, saturated
} else if depth < 0.6 {
    color = mix(c4, c3, (depth - 0.3) * 3.33);  // Transition zone
} else {
    color = c2 * 1.1;              // Cool outer — secondary, distinct
}

// GOOD: colors separated by material, not blended together
// Monolith body = c3, satellites = c4, rings = mix(c4, c2, 0.3), floor = c1
```

### Use Asymmetric Blends

When you must blend, avoid 50/50. Push toward one end:

```wgsl
// BAD
let col = mix(c2, c3, 0.5);

// GOOD: one color dominates, the other tints
let col = mix(c2, c3, 0.15);    // Mostly c2, with a hint of c3
let col = c3 * 0.85 + c4 * 0.08;  // c3 with a whisper of c4 glow
```

### Boost Saturation After Blending

If blending is unavoidable (fog, distance falloff), compensate:

```wgsl
// After a blend that may desaturate
let luminance = dot(color, vec3<f32>(0.299, 0.587, 0.114));
color = mix(vec3<f32>(luminance), color, 1.3);  // Push saturation up by 30%
```

### Emissive and Additive Color

Additive blending (using `+=` instead of `mix`) preserves vibrancy because it brightens without diluting:

```wgsl
// GOOD: additive glow keeps the accent color pure
color += c4 * glow_intensity;           // Bright accent added on top
color += c2 * fresnel * 0.3;            // Edge rim adds color, doesn't blend

// Compare to:
// BAD: mix dilutes both the base and the glow
color = mix(color, c4, glow_intensity);  // Muddies toward average
```

Use additive for glow, emissive edges, rim lighting, and god rays. Use `mix()` only for material transitions (zone boundaries, fog).

### Multiply for Tinting, Don't Average

To tint a color without dulling it:

```wgsl
// GOOD: tinting preserves saturation structure
color *= vec3<f32>(1.02, 1.0, 0.98);  // Warm shift
color *= c2 / max(max(c2.r, c2.g), c2.b);  // Normalize and tint

// BAD: mixing with a tint color toward 0.5 → dull
color = mix(color, c2, 0.3);
```

### Contrast Beats Blending

When in doubt, increase contrast instead of blending more colors into the output:

```wgsl
// Deepen contrast — this makes colors pop
color = pow(color, vec3<f32>(0.9)) * 1.2;
color = max(color - vec3<f32>(0.03), vec3<f32>(0.0));  // Crush blacks
```

A shader with three strongly separated color zones and high contrast will always look better than one that carefully blends all four colors into a smooth gradient across the screen.

## Design Principles

1. **The shader should look interesting with no audio.** Bass = 0, mid = 0, treble = 0 should still produce a moving, visually appealing image. Audio adds energy, not existence.

2. **Use all four palette colors — but keep them separated.** Assign each color a spatial or material role. Don't blend them all into the same pixel. Separation creates visual richness; averaging creates mud.

3. **Prefer slow, continuous motion.** Base animations on `iTime * 0.08`, not `iTime * 5.0`. The shader runs as a background or ambient display. Slow orbits, gentle drifts, and gradual zooms.

4. **Layer your effects.** A bare fractal or a bare starfield is less interesting than one with volumetric depth, ambient glow, and subtle foreground particles.

5. **Audio reactivity should feel natural.** Bass = large/slow things (camera shake, zoom, blob size). Mid = structure (ring thickness, spiral density). Treble = detail/sparkle (glow intensity, star brightness).

6. **Tone map and vignette.** Every shader should end with Reinhard tone mapping and a vignette. This prevents harsh clipping and gives a cinematic feel.

7. **Test with different color palettes.** A shader that looks great with blue-purple might look terrible with warm orange-red. Use `mix()` between palette colors, not hardcoded values.

8. **Contrast over blending.** If the output looks flat, the fix is almost never "blend in more colors." The fix is more contrast, stronger zone separation, or additive glow on top of a dark base.
