use bitflags::bitflags;

bitflags! {
    /// Cell attribute flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct CellFlags: u16 {
        const BOLD          = 0b0000_0001;
        const DIM           = 0b0000_0010;
        const ITALIC        = 0b0000_0100;
        const UNDERLINE     = 0b0000_1000;
        const BLINK         = 0b0001_0000;
        const INVERSE       = 0b0010_0000;
        const HIDDEN        = 0b0100_0000;
        const STRIKETHROUGH = 0b1000_0000;
    }
}

/// A single character cell in the terminal grid.
#[derive(Debug, Clone)]
pub struct Cell {
    /// The character displayed (or ' ' for empty).
    c: char,
    /// Foreground color as RGBA.
    pub fg: [u8; 4],
    /// Background color as RGBA.
    pub bg: [u8; 4],
    /// Cell attribute flags.
    pub flags: CellFlags,
}

impl Cell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn char(&self) -> char {
        self.c
    }

    pub fn set_char(&mut self, c: char) {
        self.c = c;
    }

    /// Reset cell to default state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: [255, 255, 255, 255],
            bg: [0, 0, 0, 255],
            flags: CellFlags::empty(),
        }
    }
}
