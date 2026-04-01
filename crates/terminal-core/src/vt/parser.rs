use crate::event::TerminalEvent;
use crate::grid::Grid;
use super::handler::Handler;

/// VT escape sequence parser.
///
/// Wraps the `vte` crate's state machine and dispatches parsed sequences
/// to a `Handler` which applies them to the grid.
pub struct Parser {
    state_machine: vte::Parser,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state_machine: vte::Parser::new(),
        }
    }

    /// Advance the parser by one byte.
    pub fn advance(&mut self, byte: u8, grid: &mut Grid, events: &mut Vec<TerminalEvent>) {
        let mut handler = Handler::new(grid, events);
        self.state_machine.advance(&mut handler, byte);
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
