/// A search match location in the terminal buffer.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub start_col: u16,
    pub start_row: u16,
    pub end_col: u16,
    pub end_row: u16,
}
