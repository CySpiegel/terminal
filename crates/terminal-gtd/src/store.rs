use crate::model::{Folder, Task, TaskStatus};
use rusqlite::{Connection, params};
use std::path::Path;

/// SQLite-backed task store.
pub struct TaskStore {
    conn: Connection,
}

impl TaskStore {
    /// Open or create a task database at the given path.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path).map_err(StoreError::Sqlite)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Open an in-memory database (for testing).
    pub fn in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory().map_err(StoreError::Sqlite)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Run schema migrations.
    fn migrate(&self) -> Result<(), StoreError> {
        self.conn
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS tasks (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    body TEXT,
                    folder TEXT NOT NULL DEFAULT 'inbox',
                    status TEXT NOT NULL DEFAULT 'active',
                    priority INTEGER DEFAULT 0,
                    scheduled_date TEXT,
                    due_date TEXT,
                    waiting_on TEXT,
                    tags TEXT,
                    created_at TEXT NOT NULL,
                    completed_at TEXT,
                    sort_order INTEGER DEFAULT 0
                );

                CREATE TABLE IF NOT EXISTS undo_log (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    batch_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    field TEXT NOT NULL,
                    old_value TEXT,
                    new_value TEXT,
                    timestamp TEXT NOT NULL
                );
                ",
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Insert a new task.
    pub fn insert(&self, task: &Task) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO tasks (id, title, body, folder, status, priority, scheduled_date, due_date, waiting_on, tags, created_at, completed_at, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    task.id.to_string(),
                    task.title,
                    task.body,
                    task.folder.as_str(),
                    task.status.as_str(),
                    task.priority,
                    task.scheduled_date.map(|d| d.to_string()),
                    task.due_date.map(|d| d.to_string()),
                    task.waiting_on,
                    serde_json::to_string(&task.tags).unwrap_or_default(),
                    task.created_at.to_string(),
                    task.completed_at.map(|d| d.to_string()),
                    task.sort_order,
                ],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// List all tasks in a folder.
    pub fn list_by_folder(&self, folder: &Folder) -> Result<Vec<Task>, StoreError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, body, folder, status, priority, scheduled_date, due_date, waiting_on, tags, created_at, completed_at, sort_order
                 FROM tasks WHERE folder = ?1 AND status = 'active' ORDER BY sort_order ASC",
            )
            .map_err(StoreError::Sqlite)?;

        let tasks = stmt
            .query_map(params![folder.as_str()], |row| {
                Ok(task_from_row(row))
            })
            .map_err(StoreError::Sqlite)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(tasks)
    }

    /// Delete a task by ID.
    pub fn delete(&self, id: &uuid::Uuid) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM tasks WHERE id = ?1", params![id.to_string()])
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Update a task's folder.
    pub fn move_to_folder(&self, id: &uuid::Uuid, folder: &Folder) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE tasks SET folder = ?1 WHERE id = ?2",
                params![folder.as_str(), id.to_string()],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Mark a task as done.
    pub fn complete(&self, id: &uuid::Uuid) -> Result<(), StoreError> {
        let now = chrono::Local::now().naive_local().to_string();
        self.conn
            .execute(
                "UPDATE tasks SET status = 'done', folder = 'logbook', completed_at = ?1 WHERE id = ?2",
                params![now, id.to_string()],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }
}

fn task_from_row(row: &rusqlite::Row) -> Task {
    let id_str: String = row.get_unwrap(0);
    let tags_str: String = row.get_unwrap(9);

    Task {
        id: uuid::Uuid::parse_str(&id_str).unwrap_or_default(),
        title: row.get_unwrap(1),
        body: row.get_unwrap(2),
        folder: Folder::from_str(&row.get_unwrap::<_, String>(3)),
        status: TaskStatus::from_str(&row.get_unwrap::<_, String>(4)),
        priority: row.get_unwrap(5),
        scheduled_date: row
            .get_unwrap::<_, Option<String>>(6)
            .and_then(|s| s.parse().ok()),
        due_date: row
            .get_unwrap::<_, Option<String>>(7)
            .and_then(|s| s.parse().ok()),
        waiting_on: row.get_unwrap(8),
        tags: serde_json::from_str(&tags_str).unwrap_or_default(),
        created_at: row
            .get_unwrap::<_, String>(10)
            .parse()
            .unwrap_or_default(),
        completed_at: row
            .get_unwrap::<_, Option<String>>(11)
            .and_then(|s| s.parse().ok()),
        sort_order: row.get_unwrap(12),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}
