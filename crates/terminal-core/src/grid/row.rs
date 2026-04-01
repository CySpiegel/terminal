use super::cell::Cell;

/// A single row in the terminal grid.
#[derive(Debug, Clone)]
pub struct Row {
    cells: Vec<Cell>,
}

impl Row {
    pub fn new(cols: u16) -> Self {
        Self {
            cells: (0..cols).map(|_| Cell::default()).collect(),
        }
    }

    pub fn cell(&self, col: u16) -> &Cell {
        &self.cells[col as usize]
    }

    pub fn cell_mut(&mut self, col: u16) -> &mut Cell {
        &mut self.cells[col as usize]
    }

    /// Resize the row, padding with default cells or truncating.
    pub fn resize(&mut self, cols: u16) {
        self.cells.resize(cols as usize, Cell::default());
    }

    /// Clear all cells in the row.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.reset();
        }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }
}
