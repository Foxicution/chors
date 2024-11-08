use crate::utils::PersistentIndexMap;
use chrono::{DateTime, Utc};
use rpds::HashTrieSet;
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Rc<Uuid>,
    pub description: Rc<String>,
    pub tags: HashTrieSet<String>,
    pub contexts: HashTrieSet<String>,
    pub completed: Rc<Option<DateTime<Utc>>>,
    pub subtasks: PersistentIndexMap<Uuid, Task>,
}

impl Task {
    pub fn new<S: Into<String>>(description: S) -> Self {
        let description = description.into();
        let (tags, contexts) = extract_tags_and_contexts(&description);
        Task {
            id: Uuid::now_v7().into(),
            description: description.into(),
            tags,
            contexts,
            completed: None.into(),
            subtasks: PersistentIndexMap::new(),
        }
    }

    pub fn new_with_id<S: Into<String>>(description: S, id: Uuid) -> Self {
        let description = description.into();
        let (tags, contexts) = extract_tags_and_contexts(&description);
        Task {
            id: id.into(),
            description: description.into(),
            tags,
            contexts,
            completed: None.into(),
            subtasks: PersistentIndexMap::new(),
        }
    }

    /// Update the task description immutably, extracting new tags and contexts
    pub fn with_description<S: Into<String>>(&self, new_description: S) -> Self {
        let new_description = new_description.into();
        let (tags, contexts) = extract_tags_and_contexts(&new_description);
        Task {
            description: new_description.into(),
            tags,
            contexts,
            ..self.clone()
        }
    }

    /// Flip the task and all tasks between completed/uncompleted
    pub fn with_flip_completed(&self) -> Self {
        // Determine the new completion status by flipping the current one
        let new_completed = if self.completed.is_some() {
            None // Currently completed, so unmark it
        } else {
            Some(Utc::now()) // Currently uncompleted, so mark it as completed
        };

        // Traverse and flip the completion status of tasks
        let new_subtasks =
            self.subtasks
                .keys()
                .iter()
                .fold(PersistentIndexMap::new(), |acc, key| {
                    let subtask = self.subtasks.get(key).unwrap();
                    acc.insert(*key, subtask.with_flip_completed())
                });

        Task {
            id: Rc::clone(&self.id),
            description: Rc::clone(&self.description),
            tags: self.tags.clone(),
            contexts: self.contexts.clone(),
            completed: Rc::new(new_completed),
            subtasks: new_subtasks,
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.description == other.description
            && self.tags == other.tags
            && self.contexts == other.contexts
            && self.completed == other.completed
            && self.subtasks == other.subtasks
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task() -> Task {
        Task::new("Complete the report #work @office")
    }

    fn create_test_task_with_subtasks() -> Task {
        let mut task = create_test_task();
        let subtask1 = Task::new("Write the introduction #work @home");
        let subtask2 = Task::new("Create graphs #work @office");

        task = task.with_description("Complete the report #work @office");
        task.subtasks = task.subtasks.insert(*subtask1.id, subtask1);
        task.subtasks = task.subtasks.insert(*subtask2.id, subtask2);

        task
    }

    #[test]
    fn test_new_task_with_str() {
        let task = Task::new("Complete the report #work @office");

        assert_eq!(&*task.description, "Complete the report #work @office");
        assert!(task.tags.contains("work"));
        assert!(task.contexts.contains("office"));
    }

    #[test]
    fn test_new_task_with_string() {
        let description = String::from("Finish the final draft #important @home");
        let task = Task::new(description.clone());

        assert_eq!(&*task.description, &description);
        assert!(task.tags.contains("important"));
        assert!(task.contexts.contains("home"));
    }

    #[test]
    fn test_new_task_creation() {
        let description = "Complete the report #work @office";
        let task = Task::new(description.clone());

        // Test that the task has the correct description
        assert_eq!(&*task.description, &description);

        // Test that the task has the correct tags and contexts
        assert!(task.tags.contains("work"));
        assert!(task.contexts.contains("office"));

        // Test that the task is not marked as completed
        assert!(task.completed.is_none());

        // Test that the task has no tasks
        assert_eq!(task.subtasks.len(), 0);
    }

    #[test]
    fn test_update_description() {
        let task = create_test_task();

        // Update description and check if the new description is set
        let updated_task = task.with_description("Finish the final draft #important @home");

        assert_eq!(
            &*updated_task.description,
            "Finish the final draft #important @home"
        );

        // Ensure new tags and contexts are extracted properly
        assert!(updated_task.tags.contains("important"));
        assert!(updated_task.contexts.contains("home"));

        // Ensure old tags and contexts are removed
        assert!(!updated_task.tags.contains("work"));
        assert!(!updated_task.contexts.contains("office"));

        // Ensure other fields remain unchanged
        assert_eq!(updated_task.id, task.id);
        assert_eq!(updated_task.subtasks.len(), task.subtasks.len());
    }

    #[test]
    fn test_flip_completed() {
        let task = create_test_task();

        // Initially, the task should not be completed
        assert!(task.completed.is_none());

        // Flip the completed status
        let completed_task = task.with_flip_completed();

        // Test that the task is now marked as completed
        assert!(completed_task.completed.is_some());

        // Flip the completed status again
        let uncompleted_task = completed_task.with_flip_completed();

        // Test that the task is unmarked again
        assert!(uncompleted_task.completed.is_none());
    }

    #[test]
    fn test_flip_completed_with_subtasks() {
        let task = create_test_task_with_subtasks();

        // Initially, neither the task nor its subtasks should be completed
        assert!(task.completed.is_none());
        for (_, subtask) in task.subtasks.iter() {
            assert!(subtask.completed.is_none());
        }

        // Flip the completed status
        let completed_task = task.with_flip_completed();

        // Test that both the task and its subtasks are now marked as completed
        assert!(completed_task.completed.is_some());
        for (_, subtask) in completed_task.subtasks.iter() {
            assert!(subtask.completed.is_some());
        }

        // Flip the completed status again
        let uncompleted_task = completed_task.with_flip_completed();

        // Test that both the task and its subtasks are unmarked again
        assert!(uncompleted_task.completed.is_none());
        for (_, subtask) in uncompleted_task.subtasks.iter() {
            assert!(subtask.completed.is_none());
        }
    }

    #[test]
    fn test_extract_tags_and_contexts() {
        let description = "Complete the report #work @office";
        let (tags, contexts) = extract_tags_and_contexts(&description);

        // Test that the correct tags and contexts are extracted
        assert!(tags.contains("work"));
        assert!(contexts.contains("office"));

        // Test with multiple tags and contexts
        let description = "Finish the report #important #urgent @home @work";
        let (tags, contexts) = extract_tags_and_contexts(&description);

        assert!(tags.contains("important"));
        assert!(tags.contains("urgent"));
        assert!(contexts.contains("home"));
        assert!(contexts.contains("work"));
    }

    #[test]
    fn test_display_trait() {
        let task = create_test_task();

        // Test display output for an uncompleted task
        let display = format!("{}", task);
        assert_eq!(display, "[ ] Complete the report #work @office");

        // Test display output for a completed task
        let completed_task = task.with_flip_completed();
        let display = format!("{}", completed_task);
        assert_eq!(display, "[x] Complete the report #work @office");
    }

    #[test]
    fn test_persistent_index_map_with_subtasks() {
        let task = create_test_task();
        let subtask1 = Task::new("Write the introduction #work @home");
        let subtask2 = Task::new("Create graphs #work @office");

        // Add subtasks using PersistentIndexMap
        let mut task_with_subtasks = task.clone();
        task_with_subtasks.subtasks = task_with_subtasks
            .subtasks
            .insert(*subtask1.id, subtask1.clone());
        task_with_subtasks.subtasks = task_with_subtasks
            .subtasks
            .insert(*subtask2.id, subtask2.clone());

        // Check that the subtasks were inserted correctly
        assert_eq!(task_with_subtasks.subtasks.len(), 2);
        assert!(task_with_subtasks.subtasks.contains_key(&subtask1.id));
        assert!(task_with_subtasks.subtasks.contains_key(&subtask2.id));

        // Verify the subtasks
        assert_eq!(
            task_with_subtasks.subtasks.get(&subtask1.id),
            Some(&subtask1)
        );
        assert_eq!(
            task_with_subtasks.subtasks.get(&subtask2.id),
            Some(&subtask2)
        );
    }

    #[test]
    fn test_task_equality() {
        let task1 = Task::new("Task #tag1 @context1");
        let task2 = Task::new_with_id("Task #tag1 @context1", *task1.id);

        // Even though they were created differently, they have the same content
        assert_eq!(task1, task2);
    }

    #[test]
    fn test_task_inequality() {
        let task1 = Task::new("Task1 #tag1 @context1");
        let task2 = Task::new("Task2 #tag2 @context2");

        // Different IDs and content
        assert_ne!(task1, task2);
    }

    #[test]
    fn test_extract_tags_and_contexts_no_tags_or_contexts() {
        let description = "Complete the report";
        let (tags, contexts) = extract_tags_and_contexts(description);

        assert!(tags.is_empty());
        assert!(contexts.is_empty());
    }

    #[test]
    fn test_tags_and_contexts_are_unique() {
        let description = "Task #tag #tag @context @context";
        let (tags, contexts) = extract_tags_and_contexts(description);

        let expected_tags: HashTrieSet<_> = ["tag".to_string()].into_iter().collect();
        let expected_contexts: HashTrieSet<_> = ["context".to_string()].into_iter().collect();

        assert_eq!(
            tags.iter().cloned().collect::<HashTrieSet<_>>(),
            expected_tags
        );
        assert_eq!(
            contexts.iter().cloned().collect::<HashTrieSet<_>>(),
            expected_contexts
        );
    }
}
