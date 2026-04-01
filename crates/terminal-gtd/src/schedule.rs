use crate::model::{Task, TaskStatus};
use chrono::NaiveDate;

/// Check for tasks that are due or scheduled for today.
/// Returns task IDs that should trigger notifications.
pub fn check_due_tasks(tasks: &[Task], today: NaiveDate) -> Vec<&Task> {
    tasks
        .iter()
        .filter(|t| {
            t.status == TaskStatus::Active
                && (t.due_date == Some(today) || t.scheduled_date == Some(today))
        })
        .collect()
}

/// Check for overdue tasks.
pub fn overdue_tasks(tasks: &[Task], today: NaiveDate) -> Vec<&Task> {
    tasks
        .iter()
        .filter(|t| {
            t.status == TaskStatus::Active
                && t.due_date.is_some_and(|d| d < today)
        })
        .collect()
}
