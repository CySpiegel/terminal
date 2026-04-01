/// Cursor rendering styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Beam,
    Underline,
}

/// Cursor state for rendering.
#[derive(Debug, Clone)]
pub struct CursorState {
    pub col: u16,
    pub row: u16,
    pub style: CursorStyle,
    pub visible: bool,
    pub blink_on: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            col: 0,
            row: 0,
            style: CursorStyle::Block,
            visible: true,
            blink_on: true,
        }
    }
}
