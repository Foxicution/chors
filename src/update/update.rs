use crate::{
    model::{
        filter::{Condition, FilterCondition},
        model::Model,
        task::Task,
    },
    update::message::{Direction, Message},
};
use rpds::HashTrieMap;
use std::rc::Rc;
use uuid::Uuid;

pub fn update(message: Message, model: Rc<Model>) -> Rc<Model> {
    let mut new_model = Rc::make_mut(&mut model.clone()); // Create a mutable copy of the model

    match message {
        // Task management
        Message::AddTask { task, path } => {
            let mut new_tasks = Rc::make_mut(&mut new_model.tasks).clone();
            if path.is_empty() {
                new_tasks = new_tasks.insert(task.id, task);
            } else if let Some(mut parent_task) = new_model.get_task(path).cloned() {
                parent_task.subtasks = Rc::make_mut(&mut parent_task.subtasks)
                    .insert(task.id, task)
                    .into(); // Convert to Rc<HashTrieMap>
                new_tasks = new_tasks.insert(path[0], parent_task);
            } else {
                new_model.error = Some(Rc::new(format!(
                    "Parent task not found at path {:?}",
                    &path
                )));
                return Rc::new(new_model.clone()); // Return early with error
            }

            new_model.tasks = Rc::new(new_tasks); // Update the tasks in the model
        }

        Message::RemoveTask { path } => {
            let mut new_tasks = Rc::make_mut(&mut new_model.tasks).clone();
            if path.is_empty() {
                new_model.error = Some(Rc::new(
                    "Can't remove a task with an empty path".to_string(),
                ));
            } else if let Some(_removed_task) = remove_task_at_path(&mut new_tasks, path) {
                new_model.tasks = Rc::new(new_tasks); // Update tasks if the task was removed
            } else {
                new_model.error = Some(Rc::new(format!("No task found at path {:?}", &path)));
            }
        }

        Message::UpdateTask { task_id, update } => {
            if let Some(task) = new_model.tasks.get(&task_id) {
                let mut new_task = task.clone();
                if let Some(description) = update.description {
                    new_task = new_task.update_description(description);
                }
                if let Some(completed) = update.completed {
                    new_task = match completed {
                        true => new_task.mark_completed(),
                        false => new_task.unmark_completed(),
                    };
                }
                let mut new_tasks = Rc::make_mut(&mut new_model.tasks).clone();
                new_tasks = new_tasks.insert(task_id, new_task); // Update task in tasks
                new_model.tasks = Rc::new(new_tasks); // Update model with new tasks
            } else {
                new_model.error = Some(Rc::new(format!("Task with id {} not found", task_id)));
            }
        }

        Message::MoveTask { old_path, new_path } => {
            let mut new_tasks = Rc::make_mut(&mut new_model.tasks).clone();
            if let Some(task_to_move) = remove_task_at_path(&mut new_tasks, old_path) {
                if let Some(mut new_parent_task) = new_model.get_task(new_path).cloned() {
                    new_parent_task.subtasks = Rc::make_mut(&mut new_parent_task.subtasks)
                        .insert(task_to_move.id, task_to_move)
                        .into();
                    new_tasks = new_tasks.insert(new_path[0], new_parent_task); // Update parent task
                } else {
                    new_tasks = new_tasks.insert(task_to_move.id, task_to_move);
                    // Insert as root task
                }
                new_model.tasks = Rc::new(new_tasks); // Update model with new tasks
            } else {
                new_model.error = Some(Rc::new(format!(
                    "Task not found at path {:?} for moving",
                    old_path
                )));
            }
        }

        // Filter management
        Message::AddFilter { filter } => {
            let new_filters = Rc::make_mut(&mut new_model.filters); // Use Rc::make_mut
            *new_filters = new_filters.insert(filter.id, filter); // Mutate filters in place
        }

        Message::SelectFilter { filter_id } => {
            if let Some(filter) = new_model.filters.get(filter_id) {
                new_model.current_filter = filter.filter_condition.clone();
                return update(Message::ApplyFilterCondition, Rc::new(new_model.clone()));
            // Return new model after filter is applied
            } else {
                new_model.error = Some(Rc::new(format!("Filter with id {} not found", filter_id)));
            }
        }

        Message::UpdateFilter { filter_id, update } => {
            if let Some(filter) = new_model.filters.get(&filter_id) {
                let mut new_filter = filter.clone();
                if let Some(name) = update.name {
                    new_filter.name = Rc::new(name.into()); // Convert &str to Rc<String>
                }
                if let Some(filter_condition) = update.filter_condition {
                    new_filter.filter_condition = filter_condition;
                }
                let new_filters = Rc::make_mut(&mut new_model.filters); // Use Rc::make_mut
                *new_filters = new_filters.insert(filter_id, new_filter); // Mutate filters in place
            } else {
                new_model.error = Some(Rc::new(format!("Filter with id {} not found", filter_id)));
            }
        }

        Message::UpdateCurrentFilter { expression } => {
            match FilterCondition::new(expression.to_string().into()) {
                Ok(new_filter_condition) => {
                    new_model.current_filter = Rc::new(new_filter_condition);
                }
                Err(error_message) => {
                    new_model.error = Some(Rc::new(format!("Error: {}", error_message)));
                }
            }
        }

        Message::ApplyFilterCondition => {
            let mut new_filtered_tasks = HashTrieMap::new();

            for (task_id, task) in new_model.tasks.iter() {
                let mut path = vec![*task_id];

                apply_filter(
                    task,
                    &new_model.current_filter.condition,
                    &mut new_filtered_tasks,
                    &mut path,
                    false,
                );
            }

            new_model.filtered_tasks = Rc::new(new_filtered_tasks);
        }

        // Navigation
        Message::Move { direction } => {
            let new_index = if let Some(selected_task) = new_model.selected_task {
                let old_index = new_model
                    .filtered_tasks
                    .keys()
                    .position(|&uuid| uuid == selected_task)
                    .unwrap_or(0);

                match direction {
                    Direction::Up => {
                        if old_index == 0 {
                            new_model.filtered_tasks.size() - 1
                        } else {
                            old_index - 1
                        }
                    }
                    Direction::Down => {
                        if old_index == new_model.filtered_tasks.size() - 1 {
                            0
                        } else {
                            old_index + 1
                        }
                    }
                }
            } else {
                match direction {
                    Direction::Up => 0,
                    Direction::Down => new_model.filtered_tasks.size() - 1,
                }
            };
            return update(
                Message::Select { index: new_index },
                Rc::new(new_model.clone()),
            );
        }

        Message::Select { index } => {
            if let Some((uuid, _)) = new_model.filtered_tasks.iter().nth(index) {
                new_model.selected_task = Some(*uuid);
            } else {
                new_model.error = Some(Rc::new(format!(
                    "Index {} out of range in filtered tasks",
                    index
                )));
            }
        }

        // Error
        Message::ErrorOccured { message } => {
            new_model.error = Some(Rc::new(message.to_string()));
        }
    }

    Rc::new(new_model.clone()) // Return the updated model
}

// Helper functions

fn remove_task_at_path(tasks: &mut HashTrieMap<Uuid, Task>, path: &[Uuid]) -> Option<Task> {
    if path.len() == 1 {
        tasks.get(&path[0]).cloned().map(|task| {
            *tasks = tasks.remove(&path[0]); // Remove the task from the map
            task // Return the removed task
        })
    } else if let Some(mut parent_task) = tasks.get(&path[0]).cloned() {
        let mut new_subtasks = Rc::make_mut(&mut parent_task.subtasks).clone();
        let removed_task = remove_task_at_path(&mut new_subtasks, &path[1..]);
        parent_task.subtasks = Rc::new(new_subtasks);
        *tasks = tasks.insert(path[0], parent_task);
        removed_task
    } else {
        None
    }
}

fn apply_filter(
    task: &Task,
    condition: &Condition,
    filtered_tasks: &mut HashTrieMap<Uuid, Vec<Uuid>>,
    current_path: &mut Vec<Uuid>,
    ignore_filter: bool,
) {
    let mut ignore_filter = ignore_filter;
    if ignore_filter || condition.evaluate(task) {
        *filtered_tasks = filtered_tasks.insert(task.id, current_path.clone());
        ignore_filter = true;
    }

    for (subtask_id, subtask) in task.subtasks.iter() {
        current_path.push(*subtask_id);
        apply_filter(
            subtask,
            condition,
            filtered_tasks,
            current_path,
            ignore_filter,
        );
        current_path.pop();
    }
}
