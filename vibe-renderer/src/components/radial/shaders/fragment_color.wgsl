@fragment
fn fragment_main(in: Output) -> @location(0) vec4f {
    let max_width = bar_width / 2.;
    let rel_y = abs(in.y) / max_width;

    let width_smoothing = smoothstep(1., 0., rel_y);

    // smooth out the line at the edge of the inner circle
    let bottom_smoothing = smoothstep(.0, .01, in.x);
    return color * width_smoothing * bottom_smoothing;
}