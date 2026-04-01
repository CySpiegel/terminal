use crate::grid::Grid;
use crate::vt::parser::Parser;
use crate::event::TerminalEvent;

/// Core terminal state machine.
///
/// Ties together the VT parser, character grid, and event emission.
/// Thread-safe: designed to be wrapped in `RwLock` and shared between
/// the PTY read thread (writer) and render thread (reader).
pub struct Terminal {
    grid: Grid,
    parser: Parser,
    title: String,
    cols: u16,
    rows: u16,
}

impl Terminal {
    /// Create a new terminal with the given dimensions.
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            grid: Grid::new(cols, rows),
            parser: Parser::new(),
            title: String::new(),
            cols,
            rows,
        }
    }

    /// Process raw bytes from PTY output.
    ///
    /// Parses VT escape sequences and updates the grid accordingly.
    /// Returns any events generated (bell, title change, etc.).
    pub fn process(&mut self, bytes: &[u8]) -> Vec<TerminalEvent> {
        let mut events = Vec::new();
        for byte in bytes {
            self.parser.advance(*byte, &mut self.grid, &mut events);
        }
        events
    }

    /// Resize the terminal grid.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.grid.resize(cols, rows);
    }

    /// Get a reference to the grid for rendering.
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Current terminal title (set by escape sequences).
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the terminal title (called by VT handler).
    pub(crate) fn set_title(&mut self, title: String) {
        self.title = title;
    }
}
