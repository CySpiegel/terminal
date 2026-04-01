//! Terminal character grid with scrollback.

mod cell;
mod row;
mod storage;

pub use cell::{Cell, CellFlags};
pub use row::Row;
pub use storage::Storage;

/// The terminal character grid.
///
/// Maintains visible rows plus a scrollback buffer backed by a ring buffer.
pub struct Grid {
    cols: u16,
    rows: u16,
    storage: Storage,
    cursor_col: u16,
    cursor_row: u16,
    cursor_visible: bool,
}

impl Grid {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols,
            rows,
            storage: Storage::new(cols, rows),
            cursor_col: 0,
            cursor_row: 0,
            cursor_visible: true,
        }
    }

    /// Resize the grid, reflowing content as needed.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.storage.resize(cols, rows);
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
    }

    /// Get the cell at the given position.
    pub fn cell(&self, col: u16, row: u16) -> &Cell {
        self.storage.cell(col, row)
    }

    /// Get a mutable reference to the cell at the given position.
    pub fn cell_mut(&mut self, col: u16, row: u16) -> &mut Cell {
        self.storage.cell_mut(col, row)
    }

    /// Write a character at the cursor position and advance.
    pub fn write_char(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.linefeed();
        }
        let col = self.cursor_col;
        let row = self.cursor_row;
        self.cell_mut(col, row).set_char(c);
        self.cursor_col += 1;
    }

    /// Move cursor to the next line, scrolling if needed.
    pub fn linefeed(&mut self) {
        if self.cursor_row + 1 >= self.rows {
            self.storage.scroll_up(1);
        } else {
            self.cursor_row += 1;
        }
    }

    /// Carriage return — move cursor to column 0.
    pub fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    pub fn cursor_position(&self) -> (u16, u16) {
        (self.cursor_col, self.cursor_row)
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }
}
