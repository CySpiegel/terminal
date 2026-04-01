// Cell rendering shader.
//
// Renders terminal cells as textured quads using instanced drawing.
// Each instance represents one cell with position, atlas UV, colors, and flags.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) fg_color: vec4<f32>,
    @location(2) bg_color: vec4<f32>,
    @location(3) flags: u32,
};

struct Uniforms {
    // Projection: maps grid coordinates to clip space.
    projection: mat4x4<f32>,
    // Cell dimensions in pixels.
    cell_size: vec2<f32>,
    // Atlas texture dimensions.
    atlas_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var atlas_texture: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

struct CellInstance {
    @location(0) grid_pos: vec2<f32>,
    @location(1) atlas_uv: vec4<f32>,
    @location(2) glyph_offset: vec2<f32>,
    @location(3) fg_color: vec4<f32>,
    @location(4) bg_color: vec4<f32>,
    @location(5) flags: u32,
};

// Quad vertices (two triangles).
var<private> QUAD_POSITIONS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
);

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: CellInstance,
) -> VertexOutput {
    let quad_pos = QUAD_POSITIONS[vertex_index];

    // Screen position from grid position and cell size.
    let pixel_pos = (instance.grid_pos + quad_pos) * uniforms.cell_size + instance.glyph_offset;
    let clip_pos = uniforms.projection * vec4<f32>(pixel_pos, 0.0, 1.0);

    // Atlas UV interpolation.
    let uv = instance.atlas_uv.xy + quad_pos * instance.atlas_uv.zw;

    var output: VertexOutput;
    output.position = clip_pos;
    output.uv = uv / uniforms.atlas_size;
    output.fg_color = instance.fg_color;
    output.bg_color = instance.bg_color;
    output.flags = instance.flags;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let glyph_alpha = textureSample(atlas_texture, atlas_sampler, input.uv).r;

    // Blend foreground text over background.
    let color = mix(input.bg_color, input.fg_color, glyph_alpha);
    return color;
}
