// Default audio-reactive shader for vibe web demo.
// Uses the fragment_canvas preamble uniforms:
//   iResolution, iTime, iMouse, iBPM, iColors
//   get_freq(idx) to read frequency data, FREQ_COUNT for total bins

@fragment
fn main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv = pos.xy / iResolution;
    let centered = uv - vec2f(0.5);
    // Correct for aspect ratio
    let aspect = iResolution.x / iResolution.y;
    let coord = vec2f(centered.x * aspect, centered.y);
    let dist = length(coord);
    let angle = atan2(coord.y, coord.x);

    // Audio bands
    let bass = (get_freq(0u) + get_freq(1u) + get_freq(2u) + get_freq(3u)) / 4.0;
    let mid_idx = FREQ_COUNT / 4u;
    let mid = (get_freq(mid_idx) + get_freq(mid_idx + 1u) + get_freq(mid_idx + 2u)) / 3.0;
    let hi_idx = FREQ_COUNT / 2u;
    let hi = (get_freq(hi_idx) + get_freq(hi_idx + 1u)) / 2.0;

    // Time-based rotation
    let t = iTime * 0.3;

    // Pulsing rings
    let ring1_r = 0.25 + bass * 0.2;
    let ring2_r = 0.15 + mid * 0.15;
    let ring3_r = 0.38 + hi * 0.1;

    let ring1 = smoothstep(0.025, 0.0, abs(dist - ring1_r));
    let ring2 = smoothstep(0.018, 0.0, abs(dist - ring2_r));
    let ring3 = smoothstep(0.012, 0.0, abs(dist - ring3_r));

    // Radial spokes modulated by audio
    let spoke_count = 8.0;
    let spoke_angle = angle + t;
    let spoke = pow(abs(cos(spoke_angle * spoke_count)), 40.0) * bass * 0.5;
    let spoke_mask = smoothstep(ring1_r + 0.05, ring1_r - 0.02, dist);

    // Center glow
    let glow = 0.03 / (dist + 0.01) * (0.3 + bass * 0.5);
    let glow_clamped = min(glow, 1.5);

    // Color palette - blue/purple/cyan
    let c1 = vec3f(0.2, 0.4, 1.0);  // blue
    let c2 = vec3f(0.6, 0.2, 0.9);  // purple
    let c3 = vec3f(0.1, 0.8, 0.9);  // cyan

    var color = ring1 * c1 * (1.0 + bass)
              + ring2 * c2 * (1.0 + mid)
              + ring3 * c3 * (1.0 + hi)
              + spoke * spoke_mask * mix(c1, c3, 0.5)
              + glow_clamped * mix(c1, c2, 0.5);

    // Subtle vignette
    let vignette = 1.0 - smoothstep(0.3, 0.8, dist);
    color *= vignette;

    // Mouse interaction - subtle light at cursor
    let mouse_centered = vec2f((iMouse.x - 0.5) * aspect, iMouse.y - 0.5);
    let mouse_dist = length(coord - mouse_centered);
    let mouse_glow = 0.01 / (mouse_dist + 0.01) * 0.15;
    color += vec3f(mouse_glow) * c3;

    return vec4f(clamp(color, vec3f(0.0), vec3f(1.0)), 1.0);
}
