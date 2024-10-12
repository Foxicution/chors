use crate::{
    model::{
        filter::{Condition, FilterCondition},
        model::Model,
        task::Task,
    },
    update::message::{Direction, Message},
};
use indexmap::IndexMap;
use uuid::Uuid;

pub fn update(message: Message, model: &mut Model) {
    match message {
        // Task management
        Message::AddTask { task, path } => {
            if path.is_empty() {
                model.tasks.insert(task.id, task);
            } else if let Some(parent_task) = model.get_task_mut(path) {
                parent_task.subtasks.insert(task.id, task);
            } else {
                update(
                    Message::ErrorOccured {
                        message: &format!("Parent task not found at path {:?}", &path),
                    },
                    model,
                );
            }
        }
        Message::RemoveTask { path } => {
            if path.is_empty() {
                update(
                    Message::ErrorOccured {
                        message: "Can't remove a task with an empty path",
                    },
                    model,
                );
            } else if extract_task_at_path(&mut model.tasks, path).is_none() {
                update(
                    Message::ErrorOccured {
                        message: &format!("Removal failed, no task found at {:?}", &path),
                    },
                    model,
                );
            }
        }
        Message::UpdateTask { task, update } => {
            if let Some(description) = update.description {
                task.update_description(description);
            }
            if let Some(completed) = update.completed {
                match completed {
                    true => task.mark_completed(),
                    false => task.unmark_completed(),
                };
            }
        }
        Message::MoveTask { old_path, new_path } => {
            if old_path.is_empty() {
                update(
                    Message::ErrorOccured {
                        message: "Can't move a task with an empty path",
                    },
                    model,
                );
            }
            if let Some(task) = extract_task_at_path(&mut model.tasks, old_path) {
                if let Some(parent_task) = model.get_task_mut(new_path) {
                    parent_task.subtasks.insert(task.id, task);
                } else {
                    update(Message::AddTask { task, path: &[] }, model);
                }
            } else {
                update(
                    Message::ErrorOccured {
                        message: &format!("Moving of task failed, no task found at {:?}", old_path),
                    },
                    model,
                );
            }
        }

        // Filter management
        Message::AddFilter { filter } => {
            model.filters.insert(filter.id, filter);
        }
        Message::SelectFilter { filter_id } => {
            if let Some(filter) = model.filters.get(filter_id) {
                model.current_filter = filter.filter_condition.clone();
                update(Message::ApplyFilterCondition, model);
            } else {
                update(
                    Message::ErrorOccured {
                        message: &format!("Filter with id {} not found!", filter_id),
                    },
                    model,
                );
            }
        }
        Message::UpdateFilter { filter, update } => {
            if let Some(name) = update.name {
                filter.name = name;
            }
            if let Some(filter_condition) = update.filter_condition {
                filter.filter_condition = filter_condition;
            }
        }
        Message::UpdateCurrentFilter { expression } => match FilterCondition::new(expression) {
            Ok(filter_condition) => {
                model.current_filter = filter_condition;
            }
            Err(message) => {
                update(
                    Message::ErrorOccured {
                        message: &format!("Could not parse expression\n{}", message),
                    },
                    model,
                );
            }
        },
        Message::ApplyFilterCondition => {
            model.filtered_tasks.clear();

            for (task_id, task) in &model.tasks {
                let mut path = vec![*task_id];
                apply_filter(
                    task,
                    &model.current_filter.condition,
                    &mut model.filtered_tasks,
                    &mut path,
                    false,
                );
            }
        }

        // Navigation
        Message::Move { direction } => {
            let new_index = if let Some(selected) = model.selected_task {
                let old_index = model.filtered_tasks.get_index_of(&selected).unwrap_or(0);
                match direction {
                    Direction::Up => {
                        if old_index == 0 {
                            model.filtered_tasks.len() - 1
                        } else {
                            old_index - 1
                        }
                    }
                    Direction::Down => {
                        if old_index == model.filtered_tasks.len() - 1 {
                            0
                        } else {
                            old_index + 1
                        }
                    }
                }
            } else {
                match direction {
                    Direction::Up => 0,
                    Direction::Down => model.filtered_tasks.len() - 1,
                }
            };
            update(Message::Select { index: new_index }, model);
        }

        Message::Select { index } => {
            if let Some((uuid, _)) = model.filtered_tasks.get_index(index) {
                model.selected_task = Some(*uuid);
            } else {
                update(
                    Message::ErrorOccured {
                        message: &format!(
                            "Index {} is out of range for filtered list of length {}",
                            index,
                            model.filtered_tasks.len()
                        ),
                    },
                    model,
                );
            };
        }

        // Error
        Message::ErrorOccured { message } => {
            model.error = Some(message.to_string());
        }
    }
}

// Helper functions
fn extract_task_at_path(tasks: &mut IndexMap<Uuid, Task>, path: &[Uuid]) -> Option<Task> {
    if path.len() == 1 {
        tasks.shift_remove(&path[0])
    } else if let Some(task) = tasks.get_mut(&path[0]) {
        extract_task_at_path(&mut task.subtasks, &path[1..])
    } else {
        None
    }
}

fn apply_filter(
    task: &Task,
    condition: &Condition,
    filtered_tasks: &mut IndexMap<Uuid, Vec<Uuid>>,
    current_path: &mut Vec<Uuid>,
    ignore_filter: bool,
) {
    let mut ignore_filter = ignore_filter;
    if ignore_filter || condition.evaluate(task) {
        filtered_tasks.insert(task.id, current_path.clone());
        ignore_filter = true;
    }

    for (subtask_id, subtask) in &task.subtasks {
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
