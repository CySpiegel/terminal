/// Font system wrapper around cosmic-text.
///
/// Handles font discovery, loading, shaping, and rasterization.
/// Provides the bridge between terminal cells and the glyph atlas.
pub struct FontSystem {
    font_system: cosmic_text::FontSystem,
    metrics: cosmic_text::Metrics,
}

impl FontSystem {
    /// Create a new font system with the given font size and line height.
    pub fn new(font_size: f32, line_height: f32) -> Self {
        let font_system = cosmic_text::FontSystem::new();
        let metrics = cosmic_text::Metrics::new(font_size, line_height);

        Self {
            font_system,
            metrics,
        }
    }

    /// Get the cell dimensions in pixels.
    pub fn cell_size(&self) -> (f32, f32) {
        (self.metrics.font_size * 0.6, self.metrics.line_height)
    }

    /// Access the underlying cosmic-text font system.
    pub fn inner(&mut self) -> &mut cosmic_text::FontSystem {
        &mut self.font_system
    }

    /// Update font metrics (e.g., on font size change).
    pub fn set_metrics(&mut self, font_size: f32, line_height: f32) {
        self.metrics = cosmic_text::Metrics::new(font_size, line_height);
    }

    pub fn metrics(&self) -> &cosmic_text::Metrics {
        &self.metrics
    }
}
