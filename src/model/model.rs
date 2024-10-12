use crate::model::filter::{Filter, FilterCondition};
use crate::model::task::Task;
use indexmap::IndexMap;
use uuid::Uuid;

pub struct Model {
    pub tasks: IndexMap<Uuid, Task>,

    pub filters: IndexMap<Uuid, Filter>,
    pub selected_filter_id: Option<Uuid>,
    pub current_filter: FilterCondition,

    pub filtered_tasks: IndexMap<Uuid, Vec<Uuid>>,
    pub selected_task: Option<Uuid>,
    pub error: Option<String>,
}

impl Model {
    pub fn new() -> Self {
        let empty_filter_condition = FilterCondition::new(String::new()).unwrap();
        let default_filter = Filter::new("default".to_string(), empty_filter_condition.clone());
        let default_filter_id = default_filter.id;

        Model {
            tasks: IndexMap::new(),

            filters: IndexMap::from([(default_filter_id, default_filter)]),
            selected_filter_id: Some(default_filter_id),
            current_filter: empty_filter_condition,

            filtered_tasks: IndexMap::new(),
            selected_task: None,
            error: None,
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

    pub fn get_task_mut(&mut self, path: &[Uuid]) -> Option<&mut Task> {
        let mut task = self.tasks.get_mut(&path[0]);
        for task_id in &path[1..] {
            if let Some(t) = task {
                task = t.subtasks.get_mut(task_id);
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
    use indexmap::IndexMap;
    use uuid::{NoContext, Timestamp, Uuid};

    // Helper function to create a test task
    fn create_test_task(description: &str) -> Task {
        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: description.to_string(),
            tags: vec!["work".to_string()].into_iter().collect(),
            contexts: vec!["office".to_string()].into_iter().collect(),
            completed: None,
            subtasks: IndexMap::new(),
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
        assert_eq!(model.current_filter.expression, "");
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
        assert_eq!(retrieved_task.unwrap().description, "Task 1");
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
        assert_eq!(retrieved_subtask.unwrap().description, "Subtask 1");
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

    #[test]
    fn test_get_task_mut() {
        let mut model = Model::new();
        let task = create_test_task("Mutable Task");
        let task_id = task.id;

        // Add task to root
        model.tasks.insert(task.id, task);

        // Mutably retrieve and update the task
        if let Some(task_mut) = model.get_task_mut(&[task_id]) {
            task_mut.description = "Updated Task".to_string();
        }

        // Verify the update
        let updated_task = model.get_task(&[task_id]);
        assert!(updated_task.is_some());
        assert_eq!(updated_task.unwrap().description, "Updated Task");
    }

    #[test]
    fn test_get_subtask_mut() {
        let mut model = Model::new();
        let mut parent_task = create_test_task("Parent Task");
        let subtask = create_test_task("Subtask 1");
        let subtask_id = subtask.id;

        // Add subtask to the parent task
        parent_task.subtasks.insert(subtask.id, subtask);
        let parent_task_id = parent_task.id;

        // Insert parent task into model
        model.tasks.insert(parent_task.id, parent_task);

        // Mutably retrieve and update the subtask
        if let Some(subtask_mut) = model.get_task_mut(&[parent_task_id, subtask_id]) {
            subtask_mut.description = "Updated Subtask".to_string();
        }

        // Verify the update
        let updated_subtask = model.get_task(&[parent_task_id, subtask_id]);
        assert!(updated_subtask.is_some());
        assert_eq!(updated_subtask.unwrap().description, "Updated Subtask");
    }
}
