/// Per-cell instance data sent to the GPU.
///
/// One instance per visible cell. The vertex shader positions a quad
/// from `grid_pos` and cell dimensions; the fragment shader samples
/// the glyph from the atlas.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CellInstance {
    /// Grid position (column, row) in cell units.
    pub grid_pos: [f32; 2],
    /// Atlas texture coordinates (x, y, width, height) in UV space.
    pub atlas_uv: [f32; 4],
    /// Sub-pixel glyph offset from text shaping.
    pub glyph_offset: [f32; 2],
    /// Foreground color RGBA (0.0-1.0).
    pub fg_color: [f32; 4],
    /// Background color RGBA (0.0-1.0).
    pub bg_color: [f32; 4],
    /// Attribute flags (bold, italic, underline, etc.).
    pub flags: u32,
    /// Padding for alignment.
    pub _pad: [u32; 3],
}

/// Collects cell instances for a frame.
pub struct CellBatch {
    instances: Vec<CellInstance>,
}

impl CellBatch {
    pub fn new() -> Self {
        Self {
            instances: Vec::with_capacity(4096),
        }
    }

    pub fn clear(&mut self) {
        self.instances.clear();
    }

    pub fn push(&mut self, instance: CellInstance) {
        self.instances.push(instance);
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.instances)
    }

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

impl Default for CellBatch {
    fn default() -> Self {
        Self::new()
    }
}
