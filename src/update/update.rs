use crate::{
    model::{filter::Condition, filter::Filter, model::Model, task::Task},
    update::message::Message,
    utils::reorderable_map::ReorderableMap,
};
use uuid::Uuid;

pub fn update(message: Message, model: &Model) -> Model {
    let result = match message {
        // Task management
        Message::AddSiblingTask { task } => add_sibling_task(model, task),
        Message::AddChildTask { task } => add_child_task(model, task),

        // Filter management
        Message::AddFilter { filter } => add_filter(model, filter),
        Message::SelectFilter { filter_id } => select_filter(model, filter_id),

        // Navigation
        Message::Move { direction } => Ok(model.clone()),
    };

    match result {
        Ok(new_model) => new_model,
        Err(error_message) => set_error(model, error_message),
    }
}

/// Adds a filter to the model.
fn add_filter(model: &Model, filter: Filter) -> Result<Model, String> {
    let new_filters = model.filters.insert(filter.id, filter);
    Ok(Model {
        filters: new_filters,
        ..model.clone()
    })
}

/// Selects a filter by its ID and applies it to the model.
fn select_filter(model: &Model, filter_id: Uuid) -> Result<Model, String> {
    if let Some(filter) = model.filters.get(&filter_id) {
        let new_model = Model {
            current_filter: filter.filter_condition.clone(),
            ..model.clone()
        };

        // Apply the filter to update the filtered tasks
        Ok(apply_filters_to_model(&new_model))
    } else {
        Err("Filter not found.".to_string())
    }
}

/// Adds a sibling task to the currently selected task.
fn add_sibling_task(model: &Model, task: Task) -> Result<Model, String> {
    if let Some(task_id) = model.selected_task {
        if let Some(path) = model.filtered_tasks.get(&task_id) {
            if path.len() >= 2 {
                // Add sibling to the parent task's subtasks
                let parent_path = &path[..path.len() - 1];
                if let Some(parent_task) = model.get_task(parent_path) {
                    let new_parent_task = parent_task.add_subtask(task);
                    let new_model = update_task_at_path(model, parent_path, new_parent_task)?;

                    // Reapply the filter to see if the new task should appear in the filtered view
                    let new_model = apply_filters_to_model(&new_model);

                    Ok(new_model)
                } else {
                    Err("Parent task doesn't exist.".to_string())
                }
            } else {
                // Add sibling at root level
                let new_tasks = model.tasks.insert(task.id, task);
                let new_model = Model {
                    tasks: new_tasks,
                    ..model.clone()
                };

                // Reapply the filter after adding the task
                Ok(apply_filters_to_model(&new_model))
            }
        } else {
            Err("Selected task doesn't exist.".to_string())
        }
    } else {
        // Add task at the root level if no task is selected
        let new_tasks = model.tasks.insert(task.id, task);
        let new_model = Model {
            tasks: new_tasks,
            ..model.clone()
        };

        // Reapply the filter after adding the task
        Ok(apply_filters_to_model(&new_model))
    }
}

/// Adds a child task to the currently selected task.
fn add_child_task(model: &Model, child_task: Task) -> Result<Model, String> {
    if let Some(task_id) = model.selected_task {
        if let Some(path) = model.filtered_tasks.get(&task_id) {
            if let Some(task) = model.get_task(path) {
                // Add the child task to the selected task's subtasks
                let new_task = task.add_subtask(child_task);
                let new_model = update_task_at_path(model, path, new_task)?;

                // Reapply the filter to see if the new task should appear in the filtered view
                Ok(apply_filters_to_model(&new_model))
            } else {
                Err("Selected task doesn't exist.".to_string())
            }
        } else {
            Err("Selected task path doesn't exist.".to_string())
        }
    } else {
        Err("Can't add a subtask if no tasks are selected.".to_string())
    }
}

/// Helper function to update the model's error field.
fn set_error(model: &Model, message: String) -> Model {
    Model {
        error: Some(message),
        ..model.clone()
    }
}

/// Applies the current filter to update the filtered tasks view.
fn apply_filters_to_model(model: &Model) -> Model {
    let mut filtered_tasks = ReorderableMap::new();

    for (task_id, task) in model.tasks.iter() {
        let current_path = vec![*task_id];
        filtered_tasks = apply_filter(
            task,
            &model.current_filter.condition,
            filtered_tasks,
            current_path,
            false,
        );
    }

    Model {
        filtered_tasks,
        ..model.clone()
    }
}

/// Recursive helper function to apply a filter to a task and its subtasks.
fn apply_filter(
    task: &Task,
    condition: &Condition,
    mut filtered_tasks: ReorderableMap<Uuid, Vec<Uuid>>,
    mut current_path: Vec<Uuid>,
    ignore_filter: bool,
) -> ReorderableMap<Uuid, Vec<Uuid>> {
    let new_ignore_filter = if ignore_filter || condition.evaluate(task) {
        filtered_tasks = filtered_tasks.insert(task.id, current_path.clone());
        true
    } else {
        ignore_filter
    };

    for (subtask_id, subtask) in task.subtasks.iter() {
        current_path.push(*subtask_id);
        filtered_tasks = apply_filter(
            subtask,
            condition,
            filtered_tasks,
            current_path.clone(),
            new_ignore_filter,
        );
        current_path.pop();
    }

    filtered_tasks
}

/// Updates a task at the specified path.
fn update_task_at_path(
    model: &Model,
    path: &[Uuid],
    updated_task: Task,
) -> Result<Model, &'static str> {
    if path.len() == 1 {
        let new_tasks = model.tasks.insert(path[0], updated_task);
        Ok(Model {
            tasks: new_tasks,
            ..model.clone()
        })
    } else {
        let parent_path = &path[..path.len() - 1];
        if let Some(parent_task) = model.get_task(parent_path) {
            let new_parent_task = parent_task.add_subtask(updated_task);
            update_task_at_path(model, parent_path, new_parent_task)
        } else {
            Err("Parent task doesn't exist.")
        }
    }
}

/// Adds a task at the root level of the task tree.
fn add_task_at_root(model: &Model, task: Task) -> Model {
    let new_tasks = model.tasks.insert(task.id, task);
    Model {
        tasks: new_tasks,
        ..model.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::task::Task;

    // Helper function to create a basic task for testing purposes
    fn create_task(description: &str) -> Task {
        Task::new(description.to_string())
    }

    #[test]
    fn test_add_sibling_task() {
        let model = Model::new();

        let task1 = create_task("First task");
        let task2 = create_task("Second task");

        // Add task1 to the model using the update function and a message
        let model = update(
            Message::AddSiblingTask {
                task: task1.clone(),
            },
            &model,
        );

        // Set task1 as selected
        let model = Model {
            selected_task: Some(task1.id),
            ..model
        };

        // Add task2 as a sibling to task1
        let model = update(
            Message::AddSiblingTask {
                task: task2.clone(),
            },
            &model,
        );

        // Check if both tasks are at the root level
        assert_eq!(model.tasks.len(), 2);
        assert!(model.tasks.get(&task1.id).is_some());
        assert!(model.tasks.get(&task2.id).is_some());
    }

    #[test]
    fn test_add_multiple_siblings() {
        let model = Model::new();

        let task1 = create_task("First task");
        let task2 = create_task("Second task");
        let task3 = create_task("Third task");

        // Add task1 to the model and select it
        let model = update(
            Message::AddSiblingTask {
                task: task1.clone(),
            },
            &model,
        );
        let model = Model {
            selected_task: Some(task1.id),
            ..model
        };

        // Add task2 and task3 as siblings to task1
        let model = update(
            Message::AddSiblingTask {
                task: task2.clone(),
            },
            &model,
        );
        let model = update(
            Message::AddSiblingTask {
                task: task3.clone(),
            },
            &model,
        );

        // Ensure all tasks are at the root level and are in the correct order
        assert_eq!(model.tasks.len(), 3);
        assert!(model.tasks.get(&task1.id).is_some());
        assert!(model.tasks.get(&task2.id).is_some());
        assert!(model.tasks.get(&task3.id).is_some());

        // Check order of the tasks
        let task_ids: Vec<&Uuid> = model.tasks.ordered_keys();
        assert_eq!(task_ids, vec![&task1.id, &task2.id, &task3.id]);
    }

    #[test]
    fn test_add_child_task() {
        // Initial State: Model with one parent task
        let model = Model::new();
        let parent_task = create_task("Parent task");

        // Add parent task to the model and set it as selected
        let model = update(
            Message::AddSiblingTask {
                task: parent_task.clone(),
            },
            &model,
        );

        // After adding the parent task, ensure it's in filtered_tasks
        assert!(
            model.filtered_tasks.get(&parent_task.id).is_some(),
            "Parent task not found in filtered_tasks"
        );

        // Set the parent task as selected
        let model = Model {
            selected_task: Some(parent_task.id),
            ..model
        };

        // Action: Add a child task to the parent task
        let child_task = create_task("Child task");
        let model = update(
            Message::AddChildTask {
                task: child_task.clone(),
            },
            &model,
        );

        // Expected State: Parent task should have one child
        let parent_in_model = model.get_task(&[parent_task.id]).unwrap();
        assert_eq!(
            parent_in_model.subtasks.len(),
            1,
            "Parent task should have 1 child"
        );
        assert!(
            parent_in_model.subtasks.get(&child_task.id).is_some(),
            "Child task not found in parent task's subtasks"
        );
    }

    #[test]
    fn test_add_child_to_non_existent_task() {
        let model = Model::new();

        let child_task = create_task("Child task");

        // Try adding a child task without selecting any parent task
        let result = update(
            Message::AddChildTask {
                task: child_task.clone(),
            },
            &model,
        );

        // Ensure that the operation fails with an error
        assert!(result.error.is_some());
    }

    #[test]
    fn test_update_task_at_path() {
        let model = Model::new();

        let task1 = create_task("First task");
        let updated_task1 = create_task("Updated first task");

        // Add task1 to the model
        let model = update(
            Message::AddSiblingTask {
                task: task1.clone(),
            },
            &model,
        );

        // Update task1 in the model
        let model = update_task_at_path(&model, &[task1.id], updated_task1.clone()).unwrap();

        // Verify that task1 was updated
        let updated_in_model = model.tasks.get(&task1.id).unwrap();
        assert_eq!(updated_in_model.description, updated_task1.description);
    }

    #[test]
    fn test_add_sibling_to_task_with_parent() {
        let model = Model::new();

        let parent_task = create_task("Parent task");
        let child_task = create_task("Child task");
        let sibling_task = create_task("Sibling task");

        // Add parent_task to the model
        let model = update(
            Message::AddSiblingTask {
                task: parent_task.clone(),
            },
            &model,
        );

        // Set parent_task as selected and add child_task to it
        let model = Model {
            selected_task: Some(parent_task.id),
            ..model
        };
        let model = update(
            Message::AddChildTask {
                task: child_task.clone(),
            },
            &model,
        );

        // Set child_task as selected and add sibling_task as a sibling to child_task
        let model = Model {
            selected_task: Some(child_task.id),
            ..model
        };
        let model = update(
            Message::AddSiblingTask {
                task: sibling_task.clone(),
            },
            &model,
        );

        // Verify that sibling_task was added as a sibling under the same parent
        let parent_in_model = model.tasks.get(&parent_task.id).unwrap();
        assert_eq!(parent_in_model.subtasks.len(), 2);
        assert!(parent_in_model.subtasks.get(&child_task.id).is_some());
        assert!(parent_in_model.subtasks.get(&sibling_task.id).is_some());
    }
}
