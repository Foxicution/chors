use chrono::{DateTime, Utc};
use indexmap::{IndexMap, IndexSet};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub tags: IndexSet<String>,
    pub contexts: IndexSet<String>,
    pub completed: Option<DateTime<Utc>>,
    pub subtasks: IndexMap<Uuid, Task>,
}

impl Task {
    pub fn new(description: String) -> Self {
        let mut task = Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description,
            tags: IndexSet::new(),
            contexts: IndexSet::new(),
            completed: None,
            subtasks: IndexMap::new(),
        };
        task.extract_tags_and_contexts();
        task
    }

    fn extract_tags_and_contexts(&mut self) {
        self.tags.clear();
        self.contexts.clear();
        for word in self.description.split_whitespace() {
            if let Some(tag) = word.strip_prefix('#') {
                self.tags.insert(tag.to_string());
            } else if let Some(context) = word.strip_prefix('@') {
                self.contexts.insert(context.to_string());
            }
        }
    }

    pub fn update_description(&mut self, new_description: String) {
        self.description = new_description;
        self.extract_tags_and_contexts();
    }

    pub fn mark_completed(&mut self) {
        self.completed = Some(Utc::now());
        for task in self.subtasks.values_mut() {
            task.mark_completed();
        }
    }

    pub fn unmark_completed(&mut self) {
        self.completed = None;
        for task in self.subtasks.values_mut() {
            task.unmark_completed();
        }
    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let status = if self.completed.is_some() {
            "[x]"
        } else {
            "[ ]"
        };
        write!(f, "{} {}", status, self.description)
    }
}
