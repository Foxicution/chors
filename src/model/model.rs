use std::{collections::VecDeque, rc::Rc};

use crate::model::filter::{Filter, FilterCondition};
use crate::model::task::Task;
use rpds::{HashTrieMap, Vector};
use uuid::Uuid;

pub struct Model {
    pub tasks: Rc<HashTrieMap<Uuid, Task>>,
    pub task_order: Rc<Vector<Uuid>>,

    pub filters: Rc<HashTrieMap<Uuid, Filter>>,
    pub filter_order: Rc<Vector<Uuid>>,
    pub selected_filter_id: Option<Uuid>,
    pub current_filter: Rc<FilterCondition>,

    pub filtered_tasks: Rc<HashTrieMap<Uuid, Vec<Uuid>>>,
    pub selected_task: Option<Uuid>,
    pub error: Option<Rc<String>>,

    pub undo_stack: VecDeque<Model>,
    pub redo_stack: VecDeque<Model>,
}

impl Model {
    pub fn new() -> Self {
        let empty_filter_condition = FilterCondition::new("".to_string().into()).unwrap();
        let default_filter =
            Filter::new("default".to_string().into(), empty_filter_condition.clone());
        let default_filter_id = default_filter.id;

        Model {
            tasks: Rc::new(HashTrieMap::new()),
            task_order: Rc::new(Vector::new()),

            filters: Rc::new(HashTrieMap::from_iter([(
                default_filter_id,
                default_filter,
            )])),
            filter_order: Rc::new(Vector::from_iter([default_filter_id])),
            selected_filter_id: Some(default_filter_id),
            current_filter: Rc::new(empty_filter_condition),

            filtered_tasks: Rc::new(HashTrieMap::new()),
            selected_task: None,
            error: None,

            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn get_task(&self, path: &[Uuid]) -> Option<&Task> {
        let mut task = self.tasks.get(&path[0]);
        for task_id in &path[1..] {
            if let Some(t) = task {
                task = t.subtasks.get(task_id);
            } else {
                return None;
            }
        }
        task
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::task::Task;
    use uuid::{NoContext, Timestamp, Uuid};

    // Helper function to create a test task
    fn create_test_task(description: &str) -> Task {
        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: Rc::new(description.to_string()),
            tags: Rc::new(vec!["work".to_string()].into_iter().collect()),
            contexts: Rc::new(vec!["office".to_string()].into_iter().collect()),
            completed: None,
            subtasks: Rc::new(HashTrieMap::new()),
        }
    }

    #[test]
    fn test_model_new() {
        let model = Model::new();
        assert!(model.tasks.is_empty());
        assert!(model.filters.is_empty());
        assert!(model.selected_filter_id.is_none());
        assert!(model.filtered_tasks.is_empty());
        assert!(model.selected_task.is_none());
        assert!(model.error.is_none());
        assert_eq!(model.current_filter.expression.as_ref(), "");
    }

    #[test]
    fn test_add_and_get_task_at_root() {
        let mut model = Model::new();
        let task = create_test_task("Task 1");
        let task_id = task.id;

        // Add task to root
        model.tasks.insert(task.id, task);

        // Retrieve task at root
        let retrieved_task = model.get_task(&[task_id]);
        assert!(retrieved_task.is_some());
        assert_eq!(retrieved_task.unwrap().description.as_str(), "Task 1");
    }

    #[test]
    fn test_get_non_existent_task() {
        let model = Model::new();
        let random_id = Uuid::new_v7(Timestamp::now(NoContext));

        // Try to get a task that doesn't exist
        let task = model.get_task(&[random_id]);
        assert!(task.is_none());
    }

    #[test]
    fn test_add_and_get_subtask() {
        let mut model = Model::new();
        let mut parent_task = create_test_task("Parent Task");
        let subtask = create_test_task("Subtask 1");
        let subtask_id = subtask.id;

        // Add subtask to the parent task
        parent_task.subtasks.insert(subtask.id, subtask);
        let parent_task_id = parent_task.id;

        // Insert parent task into model
        model.tasks.insert(parent_task.id, parent_task);

        // Retrieve the subtask using path
        let retrieved_subtask = model.get_task(&[parent_task_id, subtask_id]);
        assert!(retrieved_subtask.is_some());
        assert_eq!(retrieved_subtask.unwrap().description.as_str(), "Subtask 1");
    }

    #[test]
    fn test_get_non_existent_subtask() {
        let mut model = Model::new();
        let parent_task = create_test_task("Parent Task");
        let parent_task_id = parent_task.id;

        // Insert parent task without subtasks
        model.tasks.insert(parent_task_id, parent_task);

        // Try to get a non-existent subtask
        let random_id = Uuid::new_v7(Timestamp::now(NoContext));
        let task = model.get_task(&[parent_task_id, random_id]);
        assert!(task.is_none());
    }
}
