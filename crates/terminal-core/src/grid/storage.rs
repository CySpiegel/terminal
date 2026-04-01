use super::cell::Cell;
use super::row::Row;

/// Ring-buffer backed storage for the terminal grid + scrollback.
pub struct Storage {
    rows: Vec<Row>,
    cols: u16,
    visible_rows: u16,
}

impl Storage {
    pub fn new(cols: u16, visible_rows: u16) -> Self {
        let rows = (0..visible_rows).map(|_| Row::new(cols)).collect();
        Self {
            rows,
            cols,
            visible_rows,
        }
    }

    pub fn cell(&self, col: u16, row: u16) -> &Cell {
        self.rows[row as usize].cell(col)
    }

    pub fn cell_mut(&mut self, col: u16, row: u16) -> &mut Cell {
        self.rows[row as usize].cell_mut(col)
    }

    /// Scroll the visible area up by `count` lines.
    pub fn scroll_up(&mut self, count: u16) {
        for _ in 0..count {
            self.rows.remove(0);
            self.rows.push(Row::new(self.cols));
        }
    }

    /// Resize storage to new dimensions.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.visible_rows = rows;

        // Adjust row count
        while self.rows.len() < rows as usize {
            self.rows.push(Row::new(cols));
        }
        self.rows.truncate(rows as usize);

        // Adjust column count per row
        for row in &mut self.rows {
            row.resize(cols);
        }
    }
}
