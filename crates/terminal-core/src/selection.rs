/// Text selection state.
#[derive(Debug, Clone)]
pub struct Selection {
    /// Start position (col, row).
    pub start: (u16, u16),
    /// End position (col, row).
    pub end: (u16, u16),
}

