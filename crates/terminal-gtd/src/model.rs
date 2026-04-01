use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A GTD task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub body: Option<String>,
    pub folder: Folder,
    pub status: TaskStatus,
    pub priority: i32,
    pub scheduled_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub waiting_on: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::NaiveDateTime,
    pub completed_at: Option<chrono::NaiveDateTime>,
    pub sort_order: i32,
}

/// Built-in GTD folders plus user-defined custom folders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Folder {
    Inbox,
    Today,
    Upcoming,
    Waiting,
    Someday,
    Logbook,
    Custom(String),
}

impl Folder {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Inbox => "inbox",
            Self::Today => "today",
            Self::Upcoming => "upcoming",
            Self::Waiting => "waiting",
            Self::Someday => "someday",
            Self::Logbook => "logbook",
            Self::Custom(name) => name,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "inbox" => Self::Inbox,
            "today" => Self::Today,
            "upcoming" => Self::Upcoming,
            "waiting" => Self::Waiting,
            "someday" => Self::Someday,
            "logbook" => Self::Logbook,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Task lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Active,
    Done,
    Archived,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Done => "done",
            Self::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "done" => Self::Done,
            "archived" => Self::Archived,
            _ => Self::Active,
        }
    }
}

impl Task {
    /// Create a new task in the Inbox.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            body: None,
            folder: Folder::Inbox,
            status: TaskStatus::Active,
            priority: 0,
            scheduled_date: None,
            due_date: None,
            waiting_on: None,
            tags: Vec::new(),
            created_at: chrono::Local::now().naive_local(),
            completed_at: None,
            sort_order: 0,
        }
    }
}
