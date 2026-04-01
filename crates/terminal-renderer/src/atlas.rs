use std::collections::HashMap;

/// Manages the glyph texture atlas for GPU text rendering.
///
/// Glyphs are rasterized on-demand and packed into a texture using
/// etagere's shelf-packing algorithm. Cache key includes font, glyph ID,
/// size, and sub-pixel bin for crisp rendering.
pub struct GlyphAtlas {
    allocator: etagere::AtlasAllocator,
    cache: HashMap<GlyphKey, GlyphEntry>,
    texture_size: u32,
}

/// Cache key for a rasterized glyph.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GlyphKey {
    pub font_id: u16,
    pub glyph_id: u32,
    pub font_size_bits: u32,
    pub subpixel_bin: SubpixelBin,
}

/// Quantized sub-pixel position (4x1 bins).
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SubpixelBin {
    Zero,
    Quarter,
    Half,
    ThreeQuarter,
}

/// A cached glyph's location in the atlas.
#[derive(Debug, Clone, Copy)]
pub struct GlyphEntry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl GlyphAtlas {
    pub fn new(size: u32) -> Self {
        Self {
            allocator: etagere::AtlasAllocator::new(etagere::size2(size as i32, size as i32)),
            cache: HashMap::new(),
            texture_size: size,
        }
    }

    /// Look up a glyph in the cache.
    pub fn get(&self, key: &GlyphKey) -> Option<&GlyphEntry> {
        self.cache.get(key)
    }

    /// Insert a rasterized glyph into the atlas.
    pub fn insert(
        &mut self,
        key: GlyphKey,
        width: u32,
        height: u32,
        offset_x: f32,
        offset_y: f32,
    ) -> Option<GlyphEntry> {
        let alloc = self
            .allocator
            .allocate(etagere::size2(width as i32, height as i32))?;

        let entry = GlyphEntry {
            x: alloc.rectangle.min.x as u32,
            y: alloc.rectangle.min.y as u32,
            width,
            height,
            offset_x,
            offset_y,
        };

        self.cache.insert(key, entry);
        Some(entry)
    }

    /// Clear the entire atlas (e.g., on font size change).
    pub fn clear(&mut self) {
        self.allocator.clear();
        self.cache.clear();
    }

    pub fn texture_size(&self) -> u32 {
        self.texture_size
    }
}
