/// Undo/redo system using a command log pattern.
///
/// Each undoable action is recorded with the old and new values.
/// Undo replays the inverse; redo replays the forward action.

#[derive(Debug, Clone)]
pub struct UndoEntry {
    pub batch_id: String,
    pub task_id: String,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

pub struct UndoStack {
    undo: Vec<Vec<UndoEntry>>,
    redo: Vec<Vec<UndoEntry>>,
    max_size: usize,
}

impl UndoStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_size,
        }
    }

    /// Record a batch of changes as a single undoable action.
    pub fn push(&mut self, entries: Vec<UndoEntry>) {
        if self.undo.len() >= self.max_size {
            self.undo.remove(0);
        }
        self.undo.push(entries);
        self.redo.clear();
    }

    /// Pop the last action for undo. Returns the entries to reverse.
    pub fn undo(&mut self) -> Option<Vec<UndoEntry>> {
        let entries = self.undo.pop()?;
        self.redo.push(entries.clone());
        Some(entries)
    }

    /// Pop the last undone action for redo. Returns the entries to replay.
    pub fn redo(&mut self) -> Option<Vec<UndoEntry>> {
        let entries = self.redo.pop()?;
        self.undo.push(entries.clone());
        Some(entries)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
