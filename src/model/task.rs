use chrono::{DateTime, Utc};
use rpds::{HashTrieMap, HashTrieSet};
use std::rc::Rc;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub description: Rc<String>,
    pub tags: Rc<HashTrieSet<String>>,
    pub contexts: Rc<HashTrieSet<String>>,
    pub completed: Option<DateTime<Utc>>,
    pub subtasks: Rc<HashTrieMap<Uuid, Task>>,
}

impl Task {
    pub fn new(description: String) -> Self {
        let (tags, contexts) = extract_tags_and_contexts(&description);

        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: Rc::new(description),
            tags: Rc::new(tags),
            contexts: Rc::new(contexts),
            completed: None,
            subtasks: Rc::new(HashTrieMap::new()),
        }
    }

    pub fn update_description(&mut self, new_description: String) -> Task {
        let (new_tags, new_contexts) = extract_tags_and_contexts(&new_description);
        Task {
            description: Rc::new(new_description),
            tags: Rc::new(new_tags),
            contexts: Rc::new(new_contexts),
            ..self.clone()
        }
    }

    pub fn mark_completed(&self) -> Self {
        let updated_subtasks: HashTrieMap<Uuid, Task> = self
            .subtasks
            .iter()
            .map(|(id, subtask)| (*id, subtask.mark_completed()))
            .collect();

        Task {
            completed: Some(Utc::now()),
            subtasks: Rc::new(updated_subtasks),
            ..self.clone()
        }
    }

    pub fn unmark_completed(&self) -> Self {
        let updated_subtasks: HashTrieMap<Uuid, Task> = self
            .subtasks
            .iter()
            .map(|(id, subtask)| (*id, subtask.unmark_completed()))
            .collect();

        Task {
            completed: None,
            subtasks: Rc::new(updated_subtasks),
            ..self.clone()
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

// Helper function to extract tags and contexts from a description
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
