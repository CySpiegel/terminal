/// Terminal color palette.
///
/// Holds the 256 indexed colors plus default foreground/background.
pub struct ColorPalette {
    pub foreground: [f32; 4],
    pub background: [f32; 4],
    pub colors: [[f32; 4]; 256],
}

impl Default for ColorPalette {
    fn default() -> Self {
        let mut colors = [[0.0f32; 4]; 256];

        // Standard 16 colors (approximate sRGB values)
        let standard: [[f32; 4]; 16] = [
            [0.0, 0.0, 0.0, 1.0],       // Black
            [0.8, 0.0, 0.0, 1.0],       // Red
            [0.0, 0.8, 0.0, 1.0],       // Green
            [0.8, 0.8, 0.0, 1.0],       // Yellow
            [0.0, 0.0, 0.8, 1.0],       // Blue
            [0.8, 0.0, 0.8, 1.0],       // Magenta
            [0.0, 0.8, 0.8, 1.0],       // Cyan
            [0.75, 0.75, 0.75, 1.0],    // White
            [0.5, 0.5, 0.5, 1.0],       // Bright Black
            [1.0, 0.0, 0.0, 1.0],       // Bright Red
            [0.0, 1.0, 0.0, 1.0],       // Bright Green
            [1.0, 1.0, 0.0, 1.0],       // Bright Yellow
            [0.0, 0.0, 1.0, 1.0],       // Bright Blue
            [1.0, 0.0, 1.0, 1.0],       // Bright Magenta
            [0.0, 1.0, 1.0, 1.0],       // Bright Cyan
            [1.0, 1.0, 1.0, 1.0],       // Bright White
        ];
        colors[..16].copy_from_slice(&standard);

        // 216 color cube (indices 16-231)
        for r in 0..6u8 {
            for g in 0..6u8 {
                for b in 0..6u8 {
                    let idx = 16 + (r as usize * 36) + (g as usize * 6) + b as usize;
                    colors[idx] = [
                        if r == 0 { 0.0 } else { (55.0 + 40.0 * r as f32) / 255.0 },
                        if g == 0 { 0.0 } else { (55.0 + 40.0 * g as f32) / 255.0 },
                        if b == 0 { 0.0 } else { (55.0 + 40.0 * b as f32) / 255.0 },
                        1.0,
                    ];
                }
            }
        }

        // 24 grayscale (indices 232-255)
        for i in 0..24u8 {
            let v = (8.0 + 10.0 * i as f32) / 255.0;
            colors[232 + i as usize] = [v, v, v, 1.0];
        }

        Self {
            foreground: [1.0, 1.0, 1.0, 1.0],
            background: [0.0, 0.0, 0.0, 1.0],
            colors,
        }
    }
}
