// Cursor rendering shader.
//
// Renders the terminal cursor as a colored rectangle (block, beam, or underline).

struct Uniforms {
    projection: mat4x4<f32>,
    cell_size: vec2<f32>,
    cursor_pos: vec2<f32>,
    cursor_color: vec4<f32>,
    // 0 = block, 1 = beam, 2 = underline
    cursor_style: u32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
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
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var size = uniforms.cell_size;

    // Adjust size based on cursor style.
    if uniforms.cursor_style == 1u {
        // Beam: narrow width.
        size.x = 2.0;
    } else if uniforms.cursor_style == 2u {
        // Underline: thin at bottom.
        size.y = 2.0;
    }

    var offset = vec2<f32>(0.0, 0.0);
    if uniforms.cursor_style == 2u {
        offset.y = uniforms.cell_size.y - 2.0;
    }

    let pixel_pos = (uniforms.cursor_pos * uniforms.cell_size) + offset + (QUAD[vi] * size);
    let clip_pos = uniforms.projection * vec4<f32>(pixel_pos, 0.0, 1.0);

    var output: VertexOutput;
    output.position = clip_pos;
    return output;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return uniforms.cursor_color;
}
