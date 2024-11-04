use crate::{
    model::Model,
    update::{History, Message},
};

struct UpdateResult {
    model: Model,
    message: Option<String>,
    save_to_history: bool,
}

impl UpdateResult {
    fn new(model: Model) -> Self {
        Self {
            model,
            message: None,
            save_to_history: true,
        }
    }

    fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    fn without_history(mut self) -> Self {
        self.save_to_history = false;
        self
    }
}

pub fn update(message: &Message, model: &Model, history: &mut History) -> Model {
    let result = match &message {
        // Task management
        Message::AddSiblingTask { task } => model
            .with_sibling_task(task.clone())
            .map(|new_model| UpdateResult::new(new_model).with_message("Added sibling task.")),

        Message::AddChildTask { task } => model
            .with_child_task(task.clone())
            .map(|new_model| UpdateResult::new(new_model).with_message("Added child task.")),

        Message::RemoveTask { path } => model
            .with_removed_task(path)
            .map(|new_model| UpdateResult::new(new_model).with_message("Removed task.")),

        // Filter management
        Message::AddFilter { filter } => Ok(model.with_filter(filter.clone()))
            .map(|new_model| UpdateResult::new(new_model).with_message("Added new filter.")),

        Message::SelectFilter { filter_id } => model
            .with_filter_select(*filter_id)
            .map(|new_model| UpdateResult::new(new_model).with_message("Selected filter.")),

        Message::ApplyFilter { filter } => Ok(model.with_filter_condition(filter.clone()))
            .map(|new_model| UpdateResult::new(new_model).with_message("Selected filter")),

        // Navigation
        Message::Navigate { direction } => Ok(model.with_selection_moved(direction))
            .map(|new_model| UpdateResult::new(new_model).without_history()),

        // History
        Message::Undo => {
            if let Some(prev_model) = history.undo(model) {
                let last_action = history.last_action().cloned();
                let msg = match last_action {
                    Some(action) => format!("Undid action: {:?}", action),
                    None => "Undid last action.".to_string(),
                };
                Ok(UpdateResult::new(prev_model)
                    .with_message(msg)
                    .without_history())
            } else {
                Err("Nothing to undo!".to_string())
            }
        }

        Message::Redo => {
            if let Some(next_model) = history.redo(model) {
                let last_action = history.last_action().cloned();
                let msg = match last_action {
                    Some(action) => format!("Redid action: {:?}", action),
                    None => "Redid last action.".to_string(),
                };
                Ok(UpdateResult::new(next_model)
                    .with_message(msg)
                    .without_history())
            } else {
                Err("Nothing to redo!".to_string())
            }
        }
    };

    match result {
        Ok(update_result) => {
            if update_result.save_to_history {
                history.push(&update_result.model, message)
            }
            match update_result.message {
                Some(msg) => update_result.model.with_success(msg),
                None => update_result.model,
            }
        }
        Err(error_message) => model.with_error(error_message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::filter::{Filter, FilterCondition};
    use crate::model::task::Task;
    use crate::update::message::{Direction, Message};

    // Helper function to set up a model with sample tasks
    fn setup_model_with_tasks() -> Result<Model, String> {
        Model::new()
            .with_sibling_task(Task::new("Task1"))?
            .with_sibling_task(Task::new("Task2"))?
            .with_sibling_task(Task::new("Task3"))?
            .with_sibling_task(Task::new("Task4"))
    }

    #[test]
    fn test_select_filter() {
        let mut history = History::new(100);
        let mut model = setup_model_with_tasks().unwrap();

        // Add a new filter
        let filter_condition = FilterCondition::new("\"Task1\"").unwrap();
        let filter = Filter::new("Filter1", filter_condition);
        let filter_id = filter.id;
        model = model.with_filter(filter);

        // Select the new filter
        let model = update(&Message::SelectFilter { filter_id }, &model, &mut history);

        // Verify that the selected filter is applied
        assert_eq!(model.selected_filter_id.unwrap(), filter_id);
        assert_eq!(model.filtered_tasks.len(), 1);
        assert!(model
            .filtered_tasks
            .contains_key(model.tasks.get_key_at_index(0).unwrap()));
    }

    #[test]
    fn test_apply_filter() {
        let mut history = History::new(100);
        let model = setup_model_with_tasks().unwrap();

        // Create a filter condition that matches "Task2"
        let filter_condition = FilterCondition::new("\"Task2\"").unwrap();

        // Apply the filter directly
        let model = update(
            &Message::ApplyFilter {
                filter: filter_condition.clone(),
            },
            &model,
            &mut history,
        );

        // Verify that the filter is applied correctly
        assert_eq!(model.filtered_tasks.len(), 1);
        assert!(model
            .filtered_tasks
            .contains_key(model.tasks.get_key_at_index(1).unwrap()));
    }

    #[test]
    fn test_navigation_wraps_around() {
        // Test that navigation wraps around when moving past the first or last task
        let mut history = History::new(100);
        let mut model = setup_model_with_tasks().unwrap();
        model.selected_task = None;

        // Initially, no task is selected
        assert!(model.selected_task.is_none());

        // Navigate down (should select the first task)
        let model = update(
            &Message::Navigate {
                direction: Direction::Down,
            },
            &model,
            &mut history,
        );
        assert_eq!(
            model.selected_task.unwrap(),
            *model.filtered_tasks.get_key_at_index(0).unwrap()
        );

        // Navigate up from the first task (should wrap around to the last task)
        let model = update(
            &Message::Navigate {
                direction: Direction::Up,
            },
            &model,
            &mut history,
        );
        assert_eq!(
            model.selected_task.unwrap(),
            *model
                .filtered_tasks
                .get_key_at_index(model.filtered_tasks.len() - 1)
                .unwrap()
        );

        // Navigate down from the last task (should wrap around to the first task)
        let model = update(
            &Message::Navigate {
                direction: Direction::Down,
            },
            &model,
            &mut history,
        );
        assert_eq!(
            model.selected_task.unwrap(),
            *model.filtered_tasks.get_key_at_index(0).unwrap()
        );
    }

    #[test]
    fn test_add_sibling_task() {
        // Test adding a sibling task when a task is selected
        let mut history = History::new(100);
        let mut model = setup_model_with_tasks().unwrap();

        // Select task2
        model.selected_task = Some(*model.tasks.get_key_at_index(1).unwrap());

        // Add sibling task through Message
        let new_task = Task::new("New Sibling Task");
        let model = update(
            &Message::AddSiblingTask {
                task: new_task.clone(),
            },
            &model,
            &mut history,
        );

        // The new task should be added at the same level as task2
        assert!(model.tasks.contains_key(&new_task.id));
        assert_eq!(model.tasks.len(), 5);
        assert_eq!(model.selected_task.unwrap(), *new_task.id);
    }

    #[test]
    fn test_add_child_task() {
        // Test adding a child task under a selected parent
        let mut history = History::new(100);
        let mut model = setup_model_with_tasks().unwrap();

        // Select task2
        let task2_id = *model.tasks.get_key_at_index(1).unwrap();
        model.selected_task = Some(task2_id);

        // Add child task through Message
        let child_task = Task::new("Child Task");
        let model = update(
            &Message::AddChildTask {
                task: child_task.clone(),
            },
            &model,
            &mut history,
        );

        // The child task should be added under task2
        let task2 = model.tasks.get(&task2_id).unwrap();
        assert!(task2.subtasks.contains_key(&child_task.id));
        assert_eq!(task2.subtasks.len(), 1);
        assert_eq!(model.selected_task.unwrap(), *child_task.id);
    }

    #[test]
    fn test_add_child_task_no_selection() {
        // Test error handling when adding a child task with no selected task
        let mut history = History::new(100);
        let mut model = setup_model_with_tasks().unwrap();
        model.selected_task = None;

        // No task is selected
        assert!(model.selected_task.is_none());

        // Try to add a child task through Message (should result in an error)
        let child_task = Task::new("Child Task");
        let model = update(
            &Message::AddChildTask {
                task: child_task.clone(),
            },
            &model,
            &mut history,
        );

        // Should set an error in the model
        assert!(model.message.as_str().is_some());
        assert_eq!(
            model.message.as_str().unwrap(),
            "Can't insert a child task with no parent task selected"
        );
    }

    #[test]
    fn test_update_selection_on_task_removal() {
        // Test that when the selected task is removed, the selection updates to the closest task
        let mut history = History::new(100);
        let mut model = setup_model_with_tasks().unwrap();

        // Select task2
        model.selected_task = Some(*model.tasks.get_key_at_index(1).unwrap());

        // Remove task2 and verify selection update
        let model = update(
            &Message::RemoveTask {
                path: vec![model.selected_task.unwrap()],
            },
            &model,
            &mut history,
        );

        // Check that the selected task updated to the closest task
        assert!(model.selected_task.is_some());

        // The closest task should be task3, which is now at the same position task2 was in
        let expected_task_id = *model.filtered_tasks.get_key_at_index(1).unwrap();
        assert_eq!(model.selected_task.unwrap(), expected_task_id);
    }

    #[test]
    fn test_filter_tasks_with_complex_condition() {
        // Test filtering tasks with a complex condition
        let mut history = History::new(100);
        let mut model = Model::new();

        // Create tasks with various tags and contexts
        let task1 = Task::new("Task1 #work @home");
        let task2 = Task::new("Task2 #personal @gym");
        let task3 = Task::new("Task3 #work @office");
        let task4 = Task::new("Task4 #urgent @home");

        // Insert tasks into model
        model.tasks = model
            .tasks
            .insert(*task1.id, task1.clone())
            .insert(*task2.id, task2.clone())
            .insert(*task3.id, task3.clone())
            .insert(*task4.id, task4.clone());

        // Apply a complex filter
        let filter_expr = "(#work and @home) or (#urgent and not @gym)";
        let filter = FilterCondition::new(filter_expr).unwrap();

        // Update filtered_tasks through Message
        let model = update(&Message::ApplyFilter { filter }, &model, &mut history);

        // Expected to match task1 and task4
        assert!(model.filtered_tasks.contains_key(&task1.id));
        assert!(model.filtered_tasks.contains_key(&task4.id));
        assert!(!model.filtered_tasks.contains_key(&task2.id));
        assert!(!model.filtered_tasks.contains_key(&task3.id));
    }

    #[test]
    fn test_redo_functionality() {
        let model = Model::new();
        let mut history = History::new(100);
        let task = Task::new("New Task");

        // Perform an action
        let model = update(
            &Message::AddSiblingTask { task: task.clone() },
            &model,
            &mut history,
        );

        // Undo the action
        let model = update(&Message::Undo, &model, &mut history);

        // Redo the action
        let model = update(&Message::Redo, &model, &mut history);

        // Verify that the task is back in the model
        assert!(model.tasks.contains_key(&task.id));
        assert_eq!(
            model.message.as_str().unwrap(),
            format!("Redid action: AddSiblingTask {{ task: {:?} }}", task)
        );
    }

    #[test]
    fn test_undo_nothing_to_undo() {
        let model = Model::new();
        let mut history = History::new(100);

        // Attempt to undo without any history
        let model = update(&Message::Undo, &model, &mut history);

        // Verify that an error message is set
        assert_eq!(model.message.as_str().unwrap(), "Nothing to undo!");
    }

    #[test]
    fn test_redo_nothing_to_redo() {
        let model = Model::new();
        let mut history = History::new(100);

        // Attempt to redo without any history
        let model = update(&Message::Redo, &model, &mut history);

        // Verify that an error message is set
        assert_eq!(model.message.as_str().unwrap(), "Nothing to redo!");
    }

    #[test]
    fn test_add_sibling_task_with_success_message() {
        let model = Model::new();
        let mut history = History::new(100);
        let task = Task::new("New Task");

        let updated_model = update(
            &Message::AddSiblingTask { task: task.clone() },
            &model,
            &mut history,
        );

        assert_eq!(
            updated_model.message.as_str().unwrap(),
            "Added sibling task."
        );
        assert_eq!(history.undo_stack.len(), 1);
        assert_eq!(
            history.undo_stack.back().unwrap().1,
            Message::AddSiblingTask { task }
        );
    }

    #[test]
    fn test_undo_with_action_display() {
        let model = Model::new();
        let mut history = History::new(100);
        let task = Task::new("New Task");

        // Perform an action
        let model = update(
            &Message::AddSiblingTask { task: task.clone() },
            &model,
            &mut history,
        );

        // Undo the action
        let model = update(&Message::Undo, &model, &mut history);

        assert_eq!(
            model.message.as_str().unwrap(),
            format!("Undid action: AddSiblingTask {{ task: {:?} }}", task)
        );
        assert_eq!(history.undo_stack.len(), 0);
    }
}
