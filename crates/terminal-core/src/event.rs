/// Events emitted by the terminal during processing.
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// The terminal bell was triggered (BEL character).
    Bell,
    /// The window title was changed via OSC escape sequence.
    TitleChanged(String),
    /// The terminal exited (child process ended).
    Exited(i32),
}
