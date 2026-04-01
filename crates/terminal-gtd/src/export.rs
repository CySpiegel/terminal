use crate::model::Task;

/// Export tasks as JSON.
pub fn to_json(tasks: &[Task]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(tasks)
}

/// Export tasks as Markdown checklist.
pub fn to_markdown(tasks: &[Task]) -> String {
    tasks
        .iter()
        .map(|t| {
            let check = if t.status == crate::model::TaskStatus::Done {
                "x"
            } else {
                " "
            };
            format!("- [{}] {}", check, t.title)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
