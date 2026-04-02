/// Utilities for streaming LLM output to the terminal.
///
/// Handles token accumulation, line buffering, and rendering coordination
/// with the terminal's render thread.

/// Accumulates streaming tokens into complete lines for display.
pub struct TokenAccumulator {
    buffer: String,
    lines: Vec<String>,
}

impl TokenAccumulator {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            lines: Vec::new(),
        }
    }

    /// Add a token to the accumulator.
    /// Returns any complete lines that were formed.
    pub fn push(&mut self, token: &str) -> Vec<String> {
        self.buffer.push_str(token);

        let mut complete_lines = Vec::new();
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].to_string();
            complete_lines.push(line.clone());
            self.lines.push(line);
            self.buffer = self.buffer[newline_pos + 1..].to_string();
        }

        complete_lines
    }

    /// Get the current partial line (not yet terminated by newline).
    pub fn partial(&self) -> &str {
        &self.buffer
    }

    /// Flush remaining content as the final line.
    pub fn flush(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            None
        } else {
            let line = std::mem::take(&mut self.buffer);
            self.lines.push(line.clone());
            Some(line)
        }
    }

    /// Get all accumulated lines.
    pub fn all_lines(&self) -> &[String] {
        &self.lines
    }

    /// Get the full accumulated text.
    pub fn full_text(&self) -> String {
        let mut text = self.lines.join("\n");
        if !self.buffer.is_empty() {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&self.buffer);
        }
        text
    }

    /// Reset the accumulator.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.lines.clear();
    }
}

impl Default for TokenAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_accumulation() {
        let mut acc = TokenAccumulator::new();

        assert!(acc.push("Hello").is_empty());
        assert_eq!(acc.partial(), "Hello");

        let lines = acc.push(" world\nHow are");
        assert_eq!(lines, vec!["Hello world"]);
        assert_eq!(acc.partial(), "How are");

        let lines = acc.push(" you?\n");
        assert_eq!(lines, vec!["How are you?"]);
        assert_eq!(acc.partial(), "");

        assert_eq!(acc.full_text(), "Hello world\nHow are you?\n");
    }
}
