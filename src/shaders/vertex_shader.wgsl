struct VertexInput {
    @location(0) position: vec2<f32>,
}

@vertex
fn vertex_main(
    model: VertexInput,
) -> @builtin(position) vec4<f32> {
    return vec4<f32>(model.position, 0.0, 1.0);
}

