use crate::event::TerminalEvent;
use crate::grid::Grid;

/// Handles parsed VT sequences and applies them to the grid.
pub(crate) struct Handler<'a> {
    grid: &'a mut Grid,
    events: &'a mut Vec<TerminalEvent>,
}

impl<'a> Handler<'a> {
    pub fn new(grid: &'a mut Grid, events: &'a mut Vec<TerminalEvent>) -> Self {
        Self { grid, events }
    }
}

impl vte::Perform for Handler<'_> {
    fn print(&mut self, c: char) {
        self.grid.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            // BEL
            0x07 => self.events.push(TerminalEvent::Bell),
            // BS (backspace)
            0x08 => {
                let (col, _row) = self.grid.cursor_position();
                if col > 0 {
                    // Move cursor back — grid will handle via future cursor API
                }
            }
            // HT (tab) — advance to next tab stop
            0x09 => {
                let (col, _row) = self.grid.cursor_position();
                let next_tab = (col / 8 + 1) * 8;
                let target = next_tab.min(self.grid.cols() - 1);
                for _ in col..target {
                    self.grid.write_char(' ');
                }
            }
            // LF, VT, FF — linefeed
            0x0A | 0x0B | 0x0C => {
                self.grid.linefeed();
            }
            // CR — carriage return
            0x0D => {
                self.grid.carriage_return();
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS sequences — to be implemented
    }

    fn put(&mut self, _byte: u8) {
        // DCS data — to be implemented
    }

    fn unhook(&mut self) {
        // End DCS — to be implemented
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences: title changes, color queries, etc.
        if params.len() >= 2 {
            match params[0] {
                // Set window title
                b"0" | b"2" => {
                    if let Ok(title) = std::str::from_utf8(params[1]) {
                        self.events
                            .push(TerminalEvent::TitleChanged(title.to_string()));
                    }
                }
                _ => {}
            }
        }
    }

    fn csi_dispatch(
        &mut self,
        _params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
        // CSI sequences: cursor movement, erase, SGR, etc.
        // Full implementation will handle all standard CSI sequences.
        // Stub for now — this is where the bulk of VT emulation lives.
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // ESC sequences — to be implemented
    }
}
