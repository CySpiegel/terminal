// Rectangle rendering shader.
//
// Used for selection highlights, background fills, and GTD overlay backdrop.

struct Uniforms {
    projection: mat4x4<f32>,
};

struct RectInstance {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

var<private> QUAD: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
);

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    instance: RectInstance,
) -> VertexOutput {
    let pixel_pos = instance.pos + (QUAD[vi] * instance.size);
    let clip_pos = uniforms.projection * vec4<f32>(pixel_pos, 0.0, 1.0);

    var output: VertexOutput;
    output.position = clip_pos;
    output.color = instance.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
