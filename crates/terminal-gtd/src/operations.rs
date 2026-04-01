//! Pure functions for GTD operations.
//!
//! All operations take data in and return results — no side effects.
//! This makes them thread-safe and easily testable.

use crate::model::{Folder, Task, TaskStatus};
use chrono::NaiveDate;

/// Create a new task with the given title.
pub fn create_task(title: &str) -> Task {
    Task::new(title)
}

/// Move a task to a different folder.
pub fn move_task(task: &mut Task, folder: Folder) {
    task.folder = folder;
}

/// Mark a task as complete, moving it to the Logbook.
pub fn complete_task(task: &mut Task) {
    task.status = TaskStatus::Done;
    task.folder = Folder::Logbook;
    task.completed_at = Some(chrono::Local::now().naive_local());
}

/// Schedule a task for a specific date.
pub fn schedule_task(task: &mut Task, date: NaiveDate) {
    task.scheduled_date = Some(date);
    if task.folder == Folder::Inbox {
        task.folder = Folder::Upcoming;
    }
}

/// Set a due date on a task.
pub fn set_due_date(task: &mut Task, date: NaiveDate) {
    task.due_date = Some(date);
}

/// Add a tag to a task.
pub fn add_tag(task: &mut Task, tag: &str) {
    if !task.tags.contains(&tag.to_string()) {
        task.tags.push(tag.to_string());
    }
}

/// Remove a tag from a task.
pub fn remove_tag(task: &mut Task, tag: &str) {
    task.tags.retain(|t| t != tag);
}

/// Filter tasks that are due today or have today as their scheduled date.
pub fn today_tasks(tasks: &[Task], today: NaiveDate) -> Vec<&Task> {
    tasks
        .iter()
        .filter(|t| {
            t.status == TaskStatus::Active
                && (t.folder == Folder::Today
                    || t.scheduled_date == Some(today)
                    || t.due_date == Some(today))
        })
        .collect()
}

/// Filter tasks scheduled for the future.
pub fn upcoming_tasks(tasks: &[Task], today: NaiveDate) -> Vec<&Task> {
    tasks
        .iter()
        .filter(|t| {
            t.status == TaskStatus::Active
                && t.scheduled_date.is_some_and(|d| d > today)
        })
        .collect()
}

/// Filter tasks in the Waiting folder.
pub fn waiting_tasks(tasks: &[Task]) -> Vec<&Task> {
    tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Active && t.folder == Folder::Waiting)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let task = create_task("Test task");
        assert_eq!(task.title, "Test task");
        assert_eq!(task.folder, Folder::Inbox);
        assert_eq!(task.status, TaskStatus::Active);
    }

    #[test]
    fn test_complete_task() {
        let mut task = create_task("Test");
        complete_task(&mut task);
        assert_eq!(task.status, TaskStatus::Done);
        assert_eq!(task.folder, Folder::Logbook);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_schedule_moves_from_inbox() {
        let mut task = create_task("Test");
        let date = NaiveDate::from_ymd_opt(2026, 4, 15).unwrap();
        schedule_task(&mut task, date);
        assert_eq!(task.scheduled_date, Some(date));
        assert_eq!(task.folder, Folder::Upcoming);
    }

    #[test]
    fn test_add_remove_tag() {
        let mut task = create_task("Test");
        add_tag(&mut task, "urgent");
        assert!(task.tags.contains(&"urgent".to_string()));
        remove_tag(&mut task, "urgent");
        assert!(!task.tags.contains(&"urgent".to_string()));
    }
}
