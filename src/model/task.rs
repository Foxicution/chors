use crate::utils::reorderable_map::ReorderableMap;
use chrono::{DateTime, Utc};
use rpds::HashTrieSet;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub tags: HashTrieSet<String>,
    pub contexts: HashTrieSet<String>,
    pub completed: Option<DateTime<Utc>>,
    pub subtasks: ReorderableMap<Uuid, Task>,
}

impl Task {
    pub fn new(description: String) -> Self {
        let (tags, contexts) = extract_tags_and_contexts(&description);
        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description,
            tags,
            contexts,
            completed: None,
            subtasks: ReorderableMap::new(),
        }
    }

    /// Update the task description immutably, extracting new tags and contexts
    pub fn update_description(&self, new_description: String) -> Self {
        let (tags, contexts) = extract_tags_and_contexts(&new_description);
        Task {
            id: self.id,
            description: new_description,
            tags,
            contexts,
            completed: self.completed,
            subtasks: self.subtasks.clone(),
        }
    }

    /// Mark the task and all subtasks as completed (returning a new Task)
    pub fn mark_completed(&self) -> Self {
        let new_subtasks =
            self.subtasks
                .ordered_keys()
                .iter()
                .fold(ReorderableMap::new(), |acc, key| {
                    let subtask = self.subtasks.get(key).unwrap();
                    acc.insert(*key.clone(), subtask.mark_completed())
                });

        Task {
            id: self.id,
            description: self.description.clone(),
            tags: self.tags.clone(),
            contexts: self.contexts.clone(),
            completed: Some(Utc::now()),
            subtasks: new_subtasks,
        }
    }

    /// Unmark the task and all subtasks as completed (returning a new Task)
    pub fn unmark_completed(&self) -> Self {
        let new_subtasks =
            self.subtasks
                .ordered_keys()
                .iter()
                .fold(ReorderableMap::new(), |acc, key| {
                    let subtask = self.subtasks.get(key).unwrap();
                    acc.insert(*key.clone(), subtask.unmark_completed())
                });

        Task {
            id: self.id,
            description: self.description.clone(),
            tags: self.tags.clone(),
            contexts: self.contexts.clone(),
            completed: None,
            subtasks: new_subtasks,
        }
    }

    /// Add a subtask (returning a new Task with the subtask added)
    pub fn add_subtask(&self, subtask: Task) -> Self {
        let updated_subtasks = self.subtasks.insert(subtask.id, subtask);
        Task {
            id: self.id,
            description: self.description.clone(),
            tags: self.tags.clone(),
            contexts: self.contexts.clone(),
            completed: self.completed,
            subtasks: updated_subtasks,
        }
    }
}

/// Helper function to extract tags and contexts from a description
fn extract_tags_and_contexts(description: &str) -> (HashTrieSet<String>, HashTrieSet<String>) {
    let mut tags = HashTrieSet::new();
    let mut contexts = HashTrieSet::new();
    for word in description.split_whitespace() {
        if let Some(tag) = word.strip_prefix('#') {
            tags = tags.insert(tag.to_string());
        } else if let Some(context) = word.strip_prefix('@') {
            contexts = contexts.insert(context.to_string());
        }
    }
    (tags, contexts)
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
