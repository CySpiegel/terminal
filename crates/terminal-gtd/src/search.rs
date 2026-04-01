/// Full-text search over GTD tasks (placeholder for FTS5 integration).
pub fn search_tasks(query: &str, tasks: &[crate::model::Task]) -> Vec<&crate::model::Task> {
    let query_lower = query.to_lowercase();
    tasks
        .iter()
        .filter(|t| {
            t.title.to_lowercase().contains(&query_lower)
                || t.body
                    .as_ref()
                    .is_some_and(|b| b.to_lowercase().contains(&query_lower))
        })
        .collect()
}
