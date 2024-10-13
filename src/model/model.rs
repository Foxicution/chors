use crate::{
    model::{
        filter::{Filter, FilterCondition},
        task::Task,
    },
    utils::reorderable_map::ReorderableMap,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct Model {
    pub tasks: ReorderableMap<Uuid, Task>,

    pub filters: ReorderableMap<Uuid, Filter>,
    pub selected_filter_id: Option<usize>,
    pub current_filter: FilterCondition,

    pub filtered_tasks: ReorderableMap<Uuid, Vec<Uuid>>,
    pub selected_task: Option<Uuid>,

    pub error: Option<String>,
}

impl Model {
    pub fn new() -> Self {
        let empty_filter_condition = FilterCondition::new(String::new()).unwrap();
        let default_filter = Filter::new("default".to_string(), empty_filter_condition.clone());
        let default_filter_id = default_filter.id;

        Model {
            tasks: ReorderableMap::new(),

            filters: ReorderableMap::from_iter([(default_filter_id, default_filter)]),
            selected_filter_id: Some(0),
            current_filter: empty_filter_condition,

            filtered_tasks: ReorderableMap::new(),
            selected_task: None,

            error: None,
        }
    }

    /// Helper function to get a task by its path.
    pub fn get_task(&self, path: &[Uuid]) -> Option<&Task> {
        let mut task = self.tasks.get(&path[0])?;
        for task_id in &path[1..] {
            task = task.subtasks.get(task_id)?;
        }
        Some(task)
    }
}
