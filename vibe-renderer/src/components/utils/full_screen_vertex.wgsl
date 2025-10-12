const VERTICES: array<vec2f, 3> = array(
    vec2f(-3., -1.), // bottom left
    vec2f(1., -1.), // bottom right
    vec2f(1., 3.) // top right
);

@vertex
fn main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4f {
    return vec4f(VERTICES[idx], 0., 1.);
}
