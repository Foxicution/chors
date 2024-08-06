use crate::model::{Direction, Filter, FilterList, Mode, Model, Msg, Overlay, Task};
use uuid::Uuid;

pub fn update(msg: Msg, model: &mut Model) {
    match msg {
        Msg::NoOp => (),
        Msg::Quit => model.mode = Mode::Quit,
        Msg::AddTask => {
            let new_task = Task::new(&model.input);
            let new_id = new_task.id;
            let path = model.get_path();
            model.get_task_list_mut(&path).insert(new_task.id, new_task);
            model.selected = Some(new_id);
            let current_index = model.nav.get_index_of(&new_id).unwrap_or(0);
            model.list_state.select(Some(current_index));
            model.input.clear();
            model.overlay = Overlay::None;
        }
        Msg::AddSubtask => {
            let new_task = Task::new(&model.input);
            let new_id = new_task.id;
            let path = model.get_path();
            if let Some(task) = model.get_task_mut(&path) {
                task.subtasks.insert(new_task.id, new_task);
                model.selected = Some(new_id);
                let current_index = model.nav.get_index_of(&new_id).unwrap_or(0);
                model.list_state.select(Some(current_index));
                model.input.clear();
            }
            model.overlay = Overlay::None;
        }
        Msg::ToggleTaskCompletion => {
            let path = model.get_path();
            if let Some(task) = model.get_task_mut(&path) {
                task.completed = !task.completed;
                toggle_subtasks_completion(task);
                update_parent_task_completion(model, &path);
            }
        }
        Msg::SwitchMode(new_mode) => {
            model.mode = new_mode;
            model.overlay = Overlay::None;
            model.input.clear();
            model.navigation_input.clear();
            model.debug_scroll = 0;
        }
        Msg::SetOverlay(new_overlay) => {
            model.overlay = new_overlay;
            model.input.clear();
            model.navigation_input.clear();
            model.debug_scroll = 0;
        }
        Msg::NavigateTasks(direction) => {
            let nav_len = model.nav.len();
            if nav_len == 0 {
                model.selected = None;
                model.list_state.select(None);
                return;
            }

            let new_selected = match model.selected {
                Some(current) => {
                    let current_index = model.nav.get_index_of(&current).unwrap_or(0);
                    match direction {
                        Direction::Up => (current_index + nav_len - 1) % nav_len,
                        Direction::Down => (current_index + 1) % nav_len,
                    }
                }
                None => 0,
            };

            let (new_selected_id, _) = model.nav.get_index(new_selected).unwrap();
            model.selected = Some(*new_selected_id);
            model.list_state.select(Some(new_selected));
        }
        Msg::HandleNavigation => {
            if model.navigation_input.is_empty() {
                jump_to_line(model, 0);
            } else if let Ok(line) = model.navigation_input.parse::<usize>() {
                jump_to_line(model, line.saturating_sub(1));
            }
            model.overlay = Overlay::None;
            model.navigation_input.clear();
        }
        Msg::JumpToEnd => {
            if !model.nav.is_empty() {
                let last_index = model.nav.len() - 1;
                if let Some((id, _)) = model.nav.get_index(last_index) {
                    model.selected = Some(*id);
                    model.list_state.select(Some(last_index));
                }
            }
            model.overlay = Overlay::None;
            model.navigation_input.clear();
        }
        Msg::PushChar(ch) => model.input.push(ch),
        Msg::PopChar => {
            model.input.pop();
        }
        Msg::AddFilterCriterion => {
            let input = model.input.clone();
            let parts: Vec<&str> = input.split_whitespace().collect();
            let filters = parts
                .iter()
                .filter_map(|&part| {
                    if part.starts_with("completed") {
                        Some(Filter::Completed(part.ends_with("true")))
                    } else if part.starts_with("tag") {
                        Some(Filter::Tag(part[4..].to_string()))
                    } else if part.starts_with("context") {
                        Some(Filter::Context(part[8..].to_string()))
                    } else {
                        None
                    }
                })
                .collect();
            model.current_view.filter_lists.push(FilterList { filters });
            model.overlay = Overlay::None;
        }
        Msg::SaveCurrentView(view_name) => {
            model
                .saved_views
                .insert(view_name, model.current_view.clone());
        }
        Msg::LoadView(view_name) => {
            if let Some(view) = model.saved_views.get(&view_name) {
                model.current_view = view.clone();
            }
        }
        Msg::ScrollDebug(direction) => match direction {
            Direction::Up => model.debug_scroll = model.debug_scroll.saturating_sub(1),
            Direction::Down => model.debug_scroll = model.debug_scroll.saturating_add(1),
        },
    }
}

fn toggle_subtasks_completion(task: &mut Task) {
    for subtask in task.subtasks.values_mut() {
        subtask.completed = task.completed;
        toggle_subtasks_completion(subtask);
    }
}

fn jump_to_line(model: &mut Model, line: usize) {
    let max_line = model.nav.len().saturating_sub(1);
    let target_line = line.min(max_line);
    if let Some((id, _)) = model.nav.get_index(target_line) {
        model.selected = Some(*id);
        model.list_state.select(Some(target_line));
    }
}

fn update_parent_task_completion(model: &mut Model, path: &[Uuid]) {
    if path.len() <= 1 {
        return; // No parent task
    }

    let parent_path = &path[..path.len() - 1];
    if let Some(parent_task) = model.get_task_mut(parent_path) {
        let all_subtasks_completed = parent_task.subtasks.values().all(|t| t.completed);
        parent_task.completed = all_subtasks_completed;
        update_parent_task_completion(model, parent_path);
    }
}
