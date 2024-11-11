use crate::{
    model::{
        filter::{Condition, Filter, FilterCondition},
        task::Task,
    },
    update::Direction,
    utils::{PersistentIndexMap, VectorUtils},
};
use chrono::Utc;
use rpds::Vector;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum Overlay {
    AddingSiblingTask,
    AddingChildTask,
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    List,
    Quit,
}

#[derive(Clone, Debug)]
pub enum DisplayMessage {
    Success(String),
    Error(String),
    None,
}

impl DisplayMessage {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            DisplayMessage::Success(msg) | DisplayMessage::Error(msg) => Some(msg),
            DisplayMessage::None => None,
        }
    }
}

impl PartialEq for DisplayMessage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DisplayMessage::Success(msg1), DisplayMessage::Success(msg2)) => msg1 == msg2,
            (DisplayMessage::Error(msg1), DisplayMessage::Error(msg2)) => msg1 == msg2,
            (DisplayMessage::None, DisplayMessage::None) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    pub tasks: PersistentIndexMap<Uuid, Task>,

    pub filters: PersistentIndexMap<Uuid, Filter>,
    pub selected_filter_id: Option<Uuid>,
    pub current_filter: FilterCondition,

    pub filtered_tasks: PersistentIndexMap<Uuid, Vector<Uuid>>,
    pub selected_task: Option<Uuid>,

    pub message: DisplayMessage,

    pub mode: Mode,
    pub overlay: Overlay,

    pub input: String,
    pub cursor: usize,
}

impl Model {
    pub fn new() -> Self {
        let empty_filter_condition = FilterCondition::new(String::new()).unwrap();
        let default_filter = Filter::new("default", empty_filter_condition.clone());
        let default_filter_id = default_filter.id;

        Self {
            tasks: PersistentIndexMap::new(),

            filters: PersistentIndexMap::from_iter([(default_filter_id, default_filter)]),
            selected_filter_id: Some(default_filter_id),
            current_filter: empty_filter_condition,

            filtered_tasks: PersistentIndexMap::new(),
            selected_task: None,

            message: DisplayMessage::None,

            mode: Mode::List,
            overlay: Overlay::None,

            input: String::new(),
            cursor: 0,
        }
    }

    pub fn with_cursor_jump_word(&self, direction: &Direction) -> Self {
        let is_boundary = |c: char| c.is_whitespace() || c == '@' || c == '#';

        let new_cursor = match direction {
            Direction::Up => {
                let trimmed_input = &self.input[..self.cursor];

                if let Some(start_of_word) = trimmed_input.rfind(|c| !is_boundary(c)) {
                    // Check if the cursor is already at the start of the current word
                    if self.cursor > start_of_word + 1 {
                        trimmed_input[..start_of_word + 1]
                            .rfind(is_boundary)
                            .map_or(0, |i| i + 1)
                    } else {
                        // Move to the start of the previous word
                        trimmed_input[..start_of_word]
                            .rfind(is_boundary)
                            .map_or(0, |i| i + 1)
                    }
                } else {
                    0 // No previous word boundary, go to the start
                }
            }
            Direction::Down => {
                let remaining_input = &self.input[self.cursor..];

                if let Some(end_of_word) = remaining_input.find(|c| !is_boundary(c)) {
                    // Move right to the end of the current word or to the next word's start
                    let after_word_boundary = end_of_word
                        + remaining_input[end_of_word..]
                            .find(is_boundary)
                            .unwrap_or(remaining_input.len());

                    self.cursor + after_word_boundary
                } else {
                    self.input.len() // No next word boundary, go to the end
                }
            }
        };

        Self {
            cursor: new_cursor.max(0).min(self.input.len()),
            ..self.clone()
        }
    }

    pub fn with_popped_char(&self) -> Self {
        if self.cursor == 0 || self.input.is_empty() {
            return self.clone();
        }
        let mut input = self.input.clone();
        input.remove(self.cursor - 1);
        Self {
            input,
            cursor: self.cursor - 1,
            ..self.clone()
        }
    }

    pub fn with_popped_word(&self) -> Self {
        if self.cursor == 0 || self.input.is_empty() {
            return self.clone(); // If at the beginning or empty, no-op
        }
        let mut input = self.input[..self.cursor].to_string();
        let trimmed_input = input.trim_end();
        let last_space = trimmed_input.rfind(' ').unwrap_or(0);
        input.truncate(last_space);
        input.push_str(&self.input[self.cursor..]); // Preserve the text after the cursor
        Self {
            input,
            cursor: last_space,
            ..self.clone()
        }
    }

    pub fn with_inserted_char(&self, ch: char) -> Self {
        let mut input = self.input.clone();
        input.insert(self.cursor, ch);

        Self {
            input,
            cursor: self.cursor + 1,
            ..self.clone()
        }
    }

    pub fn with_move_cursor(&self, direction: &Direction) -> Self {
        let cursor = match direction {
            Direction::Down => (self.cursor + 1).min(self.input.len()),
            Direction::Up => (self.cursor - 1).max(0),
        };

        Self {
            cursor,
            ..self.clone()
        }
    }

    pub fn with_cursor(&self, position: usize) -> Self {
        Self {
            cursor: position.min(self.input.len()),
            ..self.clone()
        }
    }

    pub fn with_input(&self, input: String) -> Self {
        Self {
            input,
            ..self.clone()
        }
    }

    pub fn with_mode(&self, mode: Mode) -> Self {
        Self {
            mode,
            ..self.clone()
        }
    }

    pub fn with_overlay(&self, overlay: Overlay) -> Self {
        Self {
            overlay,
            input: String::new(),
            cursor: 0,
            ..self.clone()
        }
    }

    fn with_modified_task(
        &self,
        path: &[Uuid],
        modify_fn: fn(&Task) -> Task,
    ) -> Result<Self, String> {
        let new_tasks = modify_task_at_path(&self.tasks, path, modify_fn)?;

        let filtered_tasks = filter_tasks(&new_tasks, &self.current_filter.condition);
        let selected_task = self.get_new_selection(&filtered_tasks, None);

        Ok(Self {
            tasks: new_tasks,
            filtered_tasks,
            selected_task,
            ..self.clone()
        })
    }

    pub fn with_flipped_completion(&self, path: &[Uuid]) -> Result<Self, String> {
        let new_tasks = flip_task_and_update_parents(&self.tasks, path)?;

        let filtered_tasks = filter_tasks(&new_tasks, &self.current_filter.condition);
        let selected_task = self.get_new_selection(&filtered_tasks, None);

        Ok(Self {
            tasks: new_tasks,
            filtered_tasks,
            selected_task,
            ..self.clone()
        })
    }

    pub fn with_removed_task(&self, path: &[Uuid]) -> Result<Self, String> {
        let new_tasks = remove_task_at_path(&self.tasks, path)?;

        let filtered_tasks = filter_tasks(&new_tasks, &self.current_filter.condition);
        let selected_task = self.get_new_selection(&filtered_tasks, None);

        Ok(Self {
            tasks: new_tasks,
            filtered_tasks,
            selected_task,
            ..self.clone()
        })
    }

    pub fn with_selection_moved(&self, direction: &Direction) -> Self {
        let task_count = self.filtered_tasks.len();

        // If no tasks are filtered return the self unchanged.
        if task_count == 0 {
            return self.clone();
        }

        // If there is no selected task pick the first or last element
        if self.selected_task.is_none() {
            let selected_task = match direction {
                Direction::Up => self.filtered_tasks.get_key_at_index(task_count - 1),
                Direction::Down => self.filtered_tasks.get_key_at_index(0),
            };

            return Self {
                selected_task: selected_task.copied(),
                ..self.clone()
            };
        }

        let selected_task = self.selected_task.unwrap();
        let selected_index = self
            .filtered_tasks
            .get_index(&selected_task)
            .expect("Selected task should be in the filetered tasks");
        let new_index = match direction {
            Direction::Up => {
                if selected_index == 0 {
                    task_count - 1
                } else {
                    selected_index - 1
                }
            }
            Direction::Down => {
                if selected_index == task_count - 1 {
                    0
                } else {
                    selected_index + 1
                }
            }
        };

        Self {
            selected_task: self.filtered_tasks.get_key_at_index(new_index).cloned(),
            ..self.clone()
        }
    }

    pub fn with_sibling_task(&self, task: Task) -> Result<Self, String> {
        match self.get_path() {
            Some(path) if path.len() >= 2 => {
                let parent_path = path.drop_last().unwrap();
                let new_tasks = insert_task_and_uncomplete_parents(
                    &self.tasks,
                    &parent_path.to_vec(),
                    task.clone(),
                )?;
                Ok(self.with_tasks(new_tasks, Some(*task.id)))
            }
            _ => {
                // Adding a sibling to a root task or when no task is selected
                let mut new_tasks = self.tasks.clone();
                new_tasks = new_tasks.insert(*task.id, task.clone());

                Ok(self.with_tasks(new_tasks, Some(*task.id)))
            }
        }
    }

    pub fn with_child_task(&self, task: Task) -> Result<Self, String> {
        match self.get_path() {
            Some(path) if !path.is_empty() => {
                let new_tasks =
                    insert_task_and_uncomplete_parents(&self.tasks, &path.to_vec(), task.clone())?;
                Ok(self.with_tasks(new_tasks, Some(*task.id)))
            }
            _ => Err("Can't insert a child task with no parent task selected".to_string()),
        }
    }

    /// Updates the model with new tasks, reapplying filters and updating selection.
    fn with_tasks(
        &self,
        tasks: PersistentIndexMap<Uuid, Task>,
        desired_task_id: Option<Uuid>,
    ) -> Self {
        let filtered_tasks = filter_tasks(&tasks, &self.current_filter.condition);
        let selected_task = self.get_new_selection(&filtered_tasks, desired_task_id);

        Self {
            tasks,
            filtered_tasks,
            selected_task,
            ..self.clone()
        }
    }

    /// Updates the selected task based on the new filtered tasks.
    /// If `desired_task_id` is provided and exists in `filtered_tasks`, it is selected.
    /// Otherwise, tries to select the first task.
    /// Otherwise, tries to select the closest ancestor.
    /// Finally selects the first task if there are any tasks filtered.
    fn get_new_selection(
        &self,
        filtered_tasks: &PersistentIndexMap<Uuid, Vector<Uuid>>,
        desired_task_id: Option<Uuid>,
    ) -> Option<Uuid> {
        // If selected task is in the filtered tasks, return it
        if let Some(task_id) = desired_task_id {
            if filtered_tasks.contains_key(&task_id) {
                return Some(task_id);
            }
        }

        if let Some(selected_task_id) = self.selected_task {
            // If the old selected task is in the filtered tasks, return it
            if filtered_tasks.contains_key(&selected_task_id) {
                return Some(selected_task_id);
            // Otherwise return the closest task in the original task tree
            } else if let Some(closest_task) =
                bfs_find_closest_task(&self.tasks, filtered_tasks, selected_task_id)
            {
                return Some(closest_task);
            }
        }

        filtered_tasks.get_key_at_index(0).cloned()
    }

    pub fn with_filter(&self, filter: Filter) -> Self {
        Self {
            filters: self.filters.insert(filter.id, filter),
            ..self.clone()
        }
    }

    pub fn with_filter_condition(&self, current_filter: FilterCondition) -> Self {
        let filtered_tasks = filter_tasks(&self.tasks, &current_filter.condition);
        let selected_task = self.get_new_selection(&filtered_tasks, None);

        Self {
            filtered_tasks,
            selected_task,
            current_filter,
            ..self.clone()
        }
    }

    pub fn with_filter_select(&self, filter_id: Uuid) -> Result<Self, String> {
        if let Some(filter) = self.filters.get(&filter_id) {
            let filtered_tasks = filter_tasks(&self.tasks, &filter.filter_condition.condition);
            let selected_task = self.get_new_selection(&filtered_tasks, None);
            Ok(Self {
                selected_filter_id: Some(filter_id),
                current_filter: filter.filter_condition.clone(),
                filtered_tasks,
                selected_task,
                ..self.clone()
            })
        } else {
            Err(format!("Filter with ID {} not found.", filter_id))
        }
    }

    pub fn get_path(&self) -> Option<&Vector<Uuid>> {
        self.selected_task
            .and_then(|selected_id| self.filtered_tasks.get(&selected_id))
    }

    pub fn get_task(&self, path: &[Uuid]) -> Option<&Task> {
        if path.is_empty() {
            return None;
        };

        let mut current_tasks = &self.tasks;
        for task_id in &path[..path.len() - 1] {
            let task = current_tasks.get(task_id)?;
            current_tasks = &task.subtasks;
        }
        current_tasks.get(path.last()?)
    }

    /// Helper fonction to update the model's message field with a success message
    pub fn with_success<S: Into<String>>(&self, message: S) -> Self {
        Self {
            message: DisplayMessage::Success(message.into()),
            ..self.clone()
        }
    }

    /// Helper function to update the model's message field with an error message.
    pub fn with_error<S: Into<String>>(&self, message: S) -> Self {
        Self {
            message: DisplayMessage::Error(message.into()),
            ..self.clone()
        }
    }

    pub fn with_no_message(&self) -> Self {
        Self {
            message: DisplayMessage::None,
            ..self.clone()
        }
    }
}

fn modify_task_at_path(
    tasks: &PersistentIndexMap<Uuid, Task>,
    path: &[Uuid],
    modify_fn: fn(&Task) -> Task,
) -> Result<PersistentIndexMap<Uuid, Task>, String> {
    if path.is_empty() {
        return Err("Path is empty; cannot modify task".to_string());
    }

    let (current_id, rest_of_path) = path.split_first().expect("Path should not be empty!");

    if rest_of_path.is_empty() {
        // Base case: apply the modification function to the task at the end of the path
        if let Some(task) = tasks.get(current_id) {
            Ok(tasks.insert(*current_id, modify_fn(task)))
        } else {
            Err(format!("Task with ID {} not found", current_id))
        }
    } else {
        // Recursive case: Traverse the task hierarchy to reach the target task
        let mut current_task = tasks
            .get(current_id)
            .ok_or_else(|| format!("Task with ID {} not found", current_id))?
            .clone();

        // Recursively modify the subtask within `current_task`
        current_task.subtasks =
            modify_task_at_path(&current_task.subtasks, rest_of_path, modify_fn)?;

        // Insert the modified `current_task` back into the task map
        Ok(tasks.insert(*current_id, current_task))
    }
}

fn remove_task_at_path(
    tasks: &PersistentIndexMap<Uuid, Task>,
    path: &[Uuid],
) -> Result<PersistentIndexMap<Uuid, Task>, String> {
    // If path is empty, we cannot remove a task, so return an error
    if path.is_empty() {
        return Err("Path is empty; cannot remove task".to_string());
    }

    let (current_id, rest_of_path) = path.split_first().expect("Path should not be empty!");

    if rest_of_path.is_empty() {
        // Base case: Remove the task if it's the last element in the path
        match tasks.contains_key(current_id) {
            true => Ok(tasks.remove(current_id)),
            false => Err(format!("Task with ID {} not found", current_id)),
        }
    } else {
        // Recursive case: Locate the task in the path and remove the subtask within it
        let mut current_task = tasks
            .get(current_id)
            .ok_or_else(|| format!("Task with ID {} not found", current_id))?
            .clone();

        // Recursively update the subtasks by removing the specified subtask at the path
        current_task.subtasks = remove_task_at_path(&current_task.subtasks, rest_of_path)?;

        // Update the task map with the modified current task
        Ok(tasks.insert(*current_id, current_task))
    }
}

fn insert_task_at_path(
    tasks: &PersistentIndexMap<Uuid, Task>,
    path: &[Uuid],
    task: Task,
) -> Result<PersistentIndexMap<Uuid, Task>, String> {
    if path.is_empty() {
        Ok(tasks.insert(*task.id, task))
    } else {
        let (current_id, rest_of_path) = path.split_first().expect("Path should not be empty!");
        let mut current_task = tasks
            .get(current_id)
            .ok_or(format!("Task with ID {} not found.", current_id))?
            .clone();
        current_task.subtasks = insert_task_at_path(&current_task.subtasks, rest_of_path, task)?;
        Ok(tasks.insert(*current_task.id, current_task))
    }
}

fn bfs_find_closest_task(
    task_tree: &PersistentIndexMap<Uuid, Task>,
    filtered_tasks: &PersistentIndexMap<Uuid, Vector<Uuid>>,
    selected_task_id: Uuid,
) -> Option<Uuid> {
    let flat_task_list = flatten_tasks(task_tree);
    let selected_index = flat_task_list
        .iter()
        .position(|&id| id == selected_task_id)?;
    let mut offset = 0;
    let max_offset = flat_task_list.len().max(selected_index);
    while offset <= max_offset {
        // Check forward
        if let Some(forward_index) = selected_index.checked_add(offset) {
            if forward_index < flat_task_list.len() {
                let task_id = flat_task_list[forward_index];
                if filtered_tasks.contains_key(&task_id) {
                    return Some(task_id);
                }
            }
        }

        // Check backward
        if offset > 0 {
            if let Some(backward_index) = selected_index.checked_sub(offset) {
                let task_id = flat_task_list[backward_index];
                if filtered_tasks.contains_key(&task_id) {
                    return Some(task_id);
                }
            }
        }

        offset += 1;
    }

    None
}

fn flatten_tasks(task_tree: &PersistentIndexMap<Uuid, Task>) -> Vec<Uuid> {
    let mut result = Vec::new();
    for (task_id, task) in task_tree.iter() {
        result.push(*task_id);
        let mut subtasks = flatten_tasks(&task.subtasks);
        result.append(&mut subtasks);
    }
    result
}

fn filter_tasks(
    tasks: &PersistentIndexMap<Uuid, Task>,
    condition: &Condition,
) -> PersistentIndexMap<Uuid, Vector<Uuid>> {
    let mut results = PersistentIndexMap::new();
    let current_path = Vector::new();

    for (task_id, task) in tasks.iter() {
        filter_tasks_recursive(task_id, task, condition, &current_path, &mut results, false);
    }

    results
}

fn filter_tasks_recursive(
    task_id: &Uuid,
    task: &Task,
    condition: &Condition,
    current_path: &Vector<Uuid>,
    results: &mut PersistentIndexMap<Uuid, Vector<Uuid>>,
    parent_matches: bool,
) -> bool {
    let current_task_matches = condition.evaluate(task);
    let matches = parent_matches || current_task_matches;
    let new_path = current_path.push_back(*task_id);

    if matches {
        // Add current task to results
        *results = results.insert(*task_id, new_path.clone());
        // Include all descendants
        for (subtask_id, subtask) in task.subtasks.iter() {
            filter_tasks_include_all(subtask_id, subtask, &new_path, results);
        }
        true
    } else {
        // Check if any descendants match
        let mut any_descendants_match = false;
        for (subtask_id, subtask) in task.subtasks.iter() {
            let subtask_matches =
                filter_tasks_recursive(subtask_id, subtask, condition, &new_path, results, false);
            if subtask_matches {
                any_descendants_match = true;
            }
        }
        if any_descendants_match {
            // Include current task since a descendant matches
            *results = results.insert(*task_id, new_path);
            true
        } else {
            false
        }
    }
}

fn filter_tasks_include_all(
    task_id: &Uuid,
    task: &Task,
    current_path: &Vector<Uuid>,
    results: &mut PersistentIndexMap<Uuid, Vector<Uuid>>,
) {
    let new_path = current_path.push_back(*task_id);
    *results = results.insert(*task_id, new_path.clone());
    for (subtask_id, subtask) in task.subtasks.iter() {
        filter_tasks_include_all(subtask_id, subtask, &new_path, results);
    }
}

fn flip_task_and_update_parents(
    tasks: &PersistentIndexMap<Uuid, Task>,
    path: &[Uuid],
) -> Result<PersistentIndexMap<Uuid, Task>, String> {
    if path.is_empty() {
        return Err("Path is empty; cannot flip task completion".to_string());
    }

    let (current_id, rest_of_path) = path.split_first().unwrap();

    if let Some(task) = tasks.get(current_id) {
        if rest_of_path.is_empty() {
            // Base case: Flip the completion status of this task
            let new_task = task.with_flip_completed();
            Ok(tasks.insert(*current_id, new_task))
        } else {
            // Recursive case: Recurse into subtasks
            let new_subtasks = flip_task_and_update_parents(&task.subtasks, rest_of_path)?;

            // Update the current task's completion status based on new_subtasks
            let all_subtasks_completed = new_subtasks
                .iter()
                .all(|(_, subtask)| subtask.completed.is_some());

            let new_completed = if all_subtasks_completed {
                Some(Utc::now())
            } else {
                None
            };

            let new_task = Task {
                completed: new_completed.into(),
                subtasks: new_subtasks,
                ..task.clone()
            };

            Ok(tasks.insert(*current_id, new_task))
        }
    } else {
        Err(format!("Task with ID {} not found", current_id))
    }
}

fn insert_task_and_uncomplete_parents(
    tasks: &PersistentIndexMap<Uuid, Task>,
    path: &[Uuid],
    task_to_insert: Task,
) -> Result<PersistentIndexMap<Uuid, Task>, String> {
    if path.is_empty() {
        // Insert at the root level
        Ok(tasks.insert(*task_to_insert.id, task_to_insert))
    } else {
        let (current_id, rest_of_path) = path.split_first().unwrap();

        if let Some(current_task) = tasks.get(current_id) {
            // Recursively insert into subtasks
            let new_subtasks = insert_task_and_uncomplete_parents(
                &current_task.subtasks,
                rest_of_path,
                task_to_insert,
            )?;

            // Uncomplete the current task if it was completed
            let new_completed = if current_task.completed.is_some() {
                None
            } else {
                *current_task.completed.clone()
            };

            let new_task = Task {
                subtasks: new_subtasks,
                completed: new_completed.into(),
                ..current_task.clone()
            };

            Ok(tasks.insert(*current_id, new_task))
        } else {
            Err(format!("Task with ID {} not found", current_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use rpds::vector;

    use super::*;
    use crate::{
        model::{filter::FilterCondition, task::Task},
        persistent_map,
        update::message::Direction,
    };

    macro_rules! task_tree {
        // Task with ID, description, and subtasks
        ( $id:expr, $description:expr, [ $( $subtask:tt ),* ] )  => {{
            let mut task = Task::new_with_id($description, $id);
            #[allow(unused_mut)]
            let mut subtasks_map = PersistentIndexMap::new();
            $(
                let subtask_map = task_tree!($subtask); // Note the parentheses
                for (id, subtask) in subtask_map.iter() {
                    subtasks_map = subtasks_map.insert(*id, subtask.clone());
                }
            )*
            task.subtasks = subtasks_map;
            let mut map = PersistentIndexMap::new();
            map = map.insert(*task.id, task);
            map
        }};
        // Base case: A task with subtasks
        ( $description:expr, [ $( $subtask:tt ),* ] ) => {{
            let mut task = Task::new($description);
            #[allow(unused_mut)]
            let mut subtasks_map = PersistentIndexMap::new();
            $(
                let subtask_map = task_tree! $subtask;
                for (id, subtask) in subtask_map.iter() {
                    subtasks_map = subtasks_map.insert(*id, subtask.clone());
                }
            )*
            task.subtasks = subtasks_map;
            let mut map = PersistentIndexMap::new();
            map = map.insert(*task.id, task);
            map
        }};
        // Case for multiple root tasks
        ( $( $root_task:tt ),* ) => {{
            let mut map = PersistentIndexMap::new();
            $(
                let root_task_map = task_tree! $root_task;
                for (id, task) in root_task_map.iter() {
                    map = map.insert(*id, task.clone());
                }
            )*
            map
        }};
    }

    // Helper function to set up a model with sample tasks
    fn setup_model_with_tasks() -> Model {
        let (t1, t2, t3, t4) = (
            Task::new("Task1"),
            Task::new("Task2"),
            Task::new("Task3"),
            Task::new("Task4"),
        );
        Model {
            tasks: persistent_map! {
                *t1.id => t1.clone(),
                *t2.id => t2.clone(),
                *t3.id => t3.clone(),
                *t4.id => t4.clone()
            },
            filters: persistent_map! {},
            selected_filter_id: None,
            current_filter: FilterCondition::new("").unwrap(),
            filtered_tasks: persistent_map! {
                *t1.id => vector![*t1.id],
                *t2.id => vector![*t2.id],
                *t3.id => vector![*t3.id],
                *t4.id => vector![*t4.id]
            },
            selected_task: None,
            message: DisplayMessage::None,
            mode: Mode::List,
            overlay: Overlay::None,
            input: String::new(),
            cursor: 0,
        }
    }

    #[test]
    fn test_task_tree_with_manual_ids() {
        let (id1, id1_1, id1_1_1, id2) = (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );

        // Construct a tree with manually specified IDs
        #[rustfmt::skip]
        let task_tree = task_tree!(
            (id1, "Task1", [
                (id1_1, "Task1.1", [
                    (id1_1_1, "Task1.1.1", [])
                ])
            ]),
            (id2, "Task2", [])
        );

        // Validate the structure
        assert_eq!(task_tree.len(), 2); // Two root tasks
        assert!(task_tree.contains_key(&id1));
        assert!(task_tree.contains_key(&id2));

        // Validate "Task1" and its subtasks
        let task1 = task_tree.get(&id1).expect("Task1 should exist");
        assert_eq!(*task1.description, "Task1".to_string());
        assert!(task1.subtasks.contains_key(&id1_1));

        let task1_1 = task1.subtasks.get(&id1_1).expect("Task1.1 should exist");
        assert_eq!(*task1_1.description, "Task1.1".to_string());
        assert!(task1_1.subtasks.contains_key(&id1_1_1));

        let task1_1_1 = task1_1
            .subtasks
            .get(&id1_1_1)
            .expect("Task1.1.1 should exist");
        assert_eq!(*task1_1_1.description, "Task1.1.1".to_string());

        // Validate "Task2" without subtasks
        let task2 = task_tree.get(&id2).expect("Task2 should exist");
        assert_eq!(*task2.description, "Task2".to_string());
        assert!(task2.subtasks.is_empty());
    }

    #[test]
    fn test_task_tree_without_ids() {
        // Construct a tree without manually specified IDs
        #[rustfmt::skip]
        let task_tree = task_tree!(
            ("Task1", [
                ("Task1.1", [
                    ("Task1.1.1", [])
                ])
            ]),
            ("Task2", [])
        );

        // Validate the structure (dynamically generated IDs make exact ID testing difficult)
        assert_eq!(task_tree.len(), 2); // Two root tasks

        // Extract root task IDs and validate structure
        let root_ids: Vec<_> = task_tree.keys_to_vec();
        let task1 = task_tree
            .get(&root_ids[0])
            .expect("Root Task1 should exist");
        assert_eq!(*task1.description, "Task1".to_string());
        assert_eq!(task1.subtasks.len(), 1);

        // Check "Task1.1" and its subtask
        let task1_1_id = *task1.subtasks.keys().last().expect("Task1.1 ID");
        let task1_1 = task1
            .subtasks
            .get(&task1_1_id)
            .expect("Task1.1 should exist");
        assert_eq!(*task1_1.description, "Task1.1".to_string());

        let task1_1_1_id = *task1_1.subtasks.keys().last().expect("Task1.1.1 ID");
        let task1_1_1 = task1_1
            .subtasks
            .get(&task1_1_1_id)
            .expect("Task1.1.1 should exist");
        assert_eq!(*task1_1_1.description, "Task1.1.1".to_string());

        // Check "Task2"
        let task2 = task_tree
            .get(&root_ids[1])
            .expect("Root Task2 should exist");
        assert_eq!(*task2.description, "Task2".to_string());
        assert!(task2.subtasks.is_empty());
    }

    #[test]
    fn test_task_tree_mixed_ids() {
        let id1 = Uuid::now_v7();

        // Construct a tree with mixed manual and auto-generated IDs
        #[rustfmt::skip]
        let task_tree = task_tree!(
            (id1, "Task1", [
                ("Task1.1", [
                    ("Task1.1.1", [])
                ]),
                ("Task1.2", [])
            ]),
            ("Task2", [])
        );

        // Validate structure
        assert_eq!(task_tree.len(), 2); // Two root tasks
        assert!(task_tree.contains_key(&id1));

        // Check "Task1" and its subtasks
        let task1 = task_tree.get(&id1).expect("Task1 should exist");
        assert_eq!(*task1.description, "Task1".to_string());

        // Validate "Task1.1" and "Task1.2" under "Task1"
        assert_eq!(task1.subtasks.len(), 2);
        let mut subtask_descriptions = task1
            .subtasks
            .iter()
            .map(|(_id, t)| t.description.as_str())
            .collect::<Vec<_>>();
        subtask_descriptions.sort();
        assert_eq!(subtask_descriptions, ["Task1.1", "Task1.2"]);
    }

    #[test]
    fn test_navigation_wraps_around() {
        // Test that navigation wraps around when moving past the first or last task
        let model = setup_model_with_tasks();
        let filtered_tasks = filter_tasks(&model.tasks, &model.current_filter.condition);
        let model = Model {
            filtered_tasks,
            ..model.clone()
        };

        // Initially, no task is selected
        assert!(model.selected_task.is_none());

        // Navigate down (should select the first task)
        let model = model.with_selection_moved(&Direction::Down);
        assert_eq!(
            model.selected_task.unwrap(),
            *model.filtered_tasks.get_key_at_index(0).unwrap()
        );

        // Navigate up from the first task (should wrap around to the last task)
        let model = model.with_selection_moved(&Direction::Up);
        assert_eq!(
            model.selected_task.unwrap(),
            *model
                .filtered_tasks
                .get_key_at_index(model.filtered_tasks.len() - 1)
                .unwrap()
        );

        // Navigate down from the last task (should wrap around to the first task)
        let model = model.with_selection_moved(&Direction::Down);
        assert_eq!(
            model.selected_task.unwrap(),
            *model.filtered_tasks.get_key_at_index(0).unwrap()
        );
    }

    #[test]
    fn test_add_sibling_task() {
        // Test adding a sibling task when a task is selected
        let mut model = setup_model_with_tasks();

        // Select task2
        model.selected_task = Some(*model.tasks.get_key_at_index(1).unwrap());

        // Add sibling task
        let new_task = Task::new("New Sibling Task");
        let model = model
            .with_sibling_task(new_task.clone())
            .expect("Task is selected, adding a sibling should return no errors!");

        // The new task should be added at the same level as task2
        assert!(model.tasks.contains_key(&new_task.id));
        assert_eq!(model.tasks.len(), 5);
        assert_eq!(model.selected_task.unwrap(), *new_task.id);
    }

    #[test]
    fn test_add_child_task() {
        // Test adding a child task under a selected parent
        let mut model = setup_model_with_tasks();

        // Select task2
        let task2_id = *model
            .tasks
            .get_key_at_index(1)
            .expect("The model in this test should have tasks!");
        model.selected_task = Some(task2_id);

        // Add child task
        let child_task = Task::new("Child Task");
        let model = model
            .with_child_task(child_task.clone())
            .expect("Child adding should succeed with added task!");

        // The child task should be added under task2
        let task2 = model.tasks.get(&task2_id).unwrap();
        assert!(task2.subtasks.contains_key(&child_task.id));
        assert_eq!(task2.subtasks.len(), 1);
        assert_eq!(model.selected_task.unwrap(), *child_task.id);
    }

    #[test]
    fn test_add_child_task_no_selection() {
        // Test error handling when adding a child task with no selected task
        let model = setup_model_with_tasks();

        // No task is selected
        assert!(model.selected_task.is_none());

        // Try to add a child task (should result in an error)
        let child_task = Task::new("Child Task");
        let model = model.with_child_task(child_task);

        // Should set an error in the model
        assert!(model.is_err());
        assert_eq!(
            model.expect_err("Should be an error!"),
            "Can't insert a child task with no parent task selected"
        );
    }

    #[test]
    fn test_update_selection_on_task_removal() {
        // Test that when the selected task is removed, the selection updates to the closest task
        let model = setup_model_with_tasks();

        // Select task2
        let task2_id = *model.tasks.get_key_at_index(1).unwrap();
        let model = Model {
            selected_task: Some(task2_id),
            ..model
        };

        // Remove task2 using the `with_removed_task` method
        let model = model
            .with_removed_task(&[task2_id])
            .expect("Task removal should succeed");

        // Check that the selected task updated to the closest task
        assert!(model.selected_task.is_some());

        // The closest task should be task3, which is now at the same position task2 was in
        let expected_task_id = *model.filtered_tasks.get_key_at_index(1).unwrap();
        assert_eq!(model.selected_task.unwrap(), expected_task_id);
    }

    #[test]
    fn test_filter_tasks_with_complex_condition() {
        // Test filtering tasks with a complex condition
        let mut model = Model::new();

        // Create tasks with various tags and contexts
        let task1 = Task::new("Task1 #work @home".to_string());
        let task2 = Task::new("Task2 #personal @gym".to_string());
        let task3 = Task::new("Task3 #work @office".to_string());
        let task4 = Task::new("Task4 #urgent @home".to_string());

        // Insert tasks into model
        model.tasks = model
            .tasks
            .insert(*task1.id, task1.clone())
            .insert(*task2.id, task2.clone())
            .insert(*task3.id, task3.clone())
            .insert(*task4.id, task4.clone());

        // Apply a complex filter
        let filter_expr = "(#work and @home) or (#urgent and not @gym)";
        let filter_condition = FilterCondition::new(filter_expr.to_string()).unwrap();
        model.current_filter = filter_condition.clone();

        // Update filtered_tasks
        model.filtered_tasks = filter_tasks(&model.tasks, &model.current_filter.condition);

        // Expected to match task1 and task4
        assert!(model.filtered_tasks.contains_key(&task1.id));
        assert!(model.filtered_tasks.contains_key(&task4.id));
        assert!(!model.filtered_tasks.contains_key(&task2.id));
        assert!(!model.filtered_tasks.contains_key(&task3.id));
    }

    #[test]
    fn test_set_error() {
        // Test that the set_error function correctly sets an error in the model
        let model = Model::new();
        let error_message = "An error occurred";
        let model_with_error = model.with_error(error_message);

        assert_eq!(model_with_error.message.as_str().unwrap(), error_message);
    }

    #[test]
    fn test_flatten_tasks() {
        let (id1, id1_1, id1_1_1, id2) = (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );
        // Create a nested task tree
        #[rustfmt::skip]
        let task_tree = task_tree!(
            (id1, "Task1", [(
                id1_1, "Task1.1", [(
                    id1_1_1, "Task1.1.1", [])])]),
            (id2, "Task2", [])
        );

        // Flatten tasks
        let flat_list = flatten_tasks(&task_tree);

        // Expected order: task1, task1.1, task1.1.1, task2
        assert_eq!(flat_list.len(), 4);
        assert_eq!(flat_list[0], id1);
        assert_eq!(flat_list[1], id1_1);
        assert_eq!(flat_list[2], id1_1_1);
        assert_eq!(flat_list[3], id2);
    }

    #[test]
    fn test_bfs_find_closest_task() {
        // Test that bfs_find_closest_task finds the closest task when the selected task is not in filtered tasks
        let model = setup_model_with_tasks();

        // Let's assume filtered tasks include task1 and task3
        let filtered_tasks = PersistentIndexMap::from_iter(vec![
            (*model.tasks.get_key_at_index(0).unwrap(), Vector::new()),
            (*model.tasks.get_key_at_index(2).unwrap(), Vector::new()),
        ]);

        // Suppose selected_task_id is task2 (not in filtered_tasks)
        let selected_task_id = *model.tasks.get_key_at_index(1).unwrap();

        // Find the closest task
        let closest_task = bfs_find_closest_task(&model.tasks, &filtered_tasks, selected_task_id);

        // The closest tasks are task1 and task3; task3 is the next task after task2
        assert_eq!(
            closest_task.unwrap(),
            *model.tasks.get_key_at_index(2).unwrap()
        );
    }

    #[test]
    fn test_insert_task_at_path() {
        // Test inserting a task at a given path in the task tree
        let mut tasks = PersistentIndexMap::new();

        // Create tasks
        let task1 = Task::new("Task1".to_string());
        let task2 = Task::new("Task2".to_string());
        let task3 = Task::new("Task3".to_string());

        // Insert task1
        tasks = tasks.insert(*task1.id, task1.clone());

        // Insert task2 under task1
        tasks = insert_task_at_path(&tasks, &[*task1.id], task2.clone()).unwrap();

        // Insert task3 under task1 -> task2
        tasks = insert_task_at_path(&tasks, &[*task1.id, *task2.id], task3.clone()).unwrap();

        // Verify the structure
        let task1 = tasks.get(&task1.id).unwrap();
        let task2 = task1.subtasks.get(&task2.id).unwrap();
        let task3 = task2.subtasks.get(&task3.id).unwrap();

        assert_eq!(*task3.id, *task3.id);
    }

    #[test]
    fn test_insert_task_at_invalid_path() {
        // Test error handling when trying to insert a task at an invalid path
        let mut tasks = PersistentIndexMap::new();

        // Create tasks
        let task1 = Task::new("Task1".to_string());
        let task2 = Task::new("Task2".to_string());

        // Insert task1
        tasks = tasks.insert(*task1.id, task1.clone());

        // Try to insert task2 under a non-existent path
        let result = insert_task_at_path(&tasks, &[Uuid::now_v7()], task2.clone());

        assert!(result.is_err());
    }

    #[test]
    fn test_remove_task_at_path() {
        // Create a nested task tree
        let (id1, id1_1, id1_1_1, id2) = (
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
            Uuid::now_v7(),
        );

        // Construct a tree with nested tasks
        #[rustfmt::skip]
        let task_tree = task_tree!(
            (id1, "Task1", [
                (id1_1, "Task1.1", [
                    (id1_1_1, "Task1.1.1", [])
                ])
            ]),
            (id2, "Task2", [])
        );

        // Remove Task1.1.1 from Task1 -> Task1.1 -> Task1.1.1
        let new_task_tree = remove_task_at_path(&task_tree, &[id1, id1_1, id1_1_1])
            .expect("Task should be removed successfully");

        // Task1 should still exist
        assert!(new_task_tree.contains_key(&id1));

        // Task1.1.1 should be removed from Task1.1
        let task1 = new_task_tree.get(&id1).unwrap();
        let task1_1 = task1.subtasks.get(&id1_1).unwrap();
        assert!(!task1_1.subtasks.contains_key(&id1_1_1));
    }

    #[test]
    fn test_remove_task_with_invalid_path() {
        // Create a task tree with one task
        let (id1, id2) = (Uuid::now_v7(), Uuid::now_v7());
        assert!(id1 != id2, "Id's must not be equal");
        let task_tree = task_tree!((id1, "Task1", []));

        // Attempt to remove a task using an invalid path
        let result = remove_task_at_path(&task_tree, &[id2]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Task with ID {} not found", id2)
        );
    }

    #[test]
    fn test_with_removed_task() {
        // Setup a model with tasks
        let model = setup_model_with_tasks();

        // Select task1 for removal
        let path = vec![*model.tasks.get_key_at_index(0).unwrap()];
        let model = model
            .with_removed_task(&path)
            .expect("Removal should succeed");

        // Verify task1 has been removed
        assert_eq!(model.tasks.len(), 3);
        assert!(!model.tasks.contains_key(&path[0]));
    }

    #[test]
    fn test_with_filter() {
        // Setup a model and add a new filter
        let model = Model::new();
        let filter = Filter::new("New Filter", FilterCondition::new("#urgent").unwrap());
        let model = model.with_filter(filter.clone());

        // Verify the new filter is added
        assert!(model.filters.contains_key(&filter.id));
        assert_eq!(model.filters.len(), 2); // Including the default filter
    }

    #[test]
    fn test_with_filter_condition() {
        // Setup a model with tasks
        let model = setup_model_with_tasks();

        // Define a filter condition that matches "Task1" only
        let filter_condition = FilterCondition::new("\"Task1\"").unwrap();

        // Apply the filter condition
        let model = model.with_filter_condition(filter_condition.clone());

        // Verify that only "Task1" is in the filtered tasks
        assert_eq!(model.filtered_tasks.len(), 1);
        assert!(model
            .filtered_tasks
            .contains_key(model.tasks.get_key_at_index(0).unwrap()));
    }

    #[test]
    fn test_with_filter_select_valid() {
        // Setup a model with a custom filter
        let model = setup_model_with_tasks();
        let filter = Filter::new("Custom Filter", FilterCondition::new("\"Task1\"").unwrap());
        let filter_id = filter.id;
        let model = model
            .with_filter(filter)
            .with_filter_select(filter_id)
            .unwrap();

        // Verify selected filter and filtered tasks
        assert_eq!(model.selected_filter_id.unwrap(), filter_id);
        assert!(model.filtered_tasks.contains_key(
            model
                .tasks
                .get_key_at_index(0)
                .expect("First task should exist")
        ));
    }

    #[test]
    fn test_with_filter_select_invalid() {
        // Setup a model with no custom filters
        let model = setup_model_with_tasks();

        // Try selecting a non-existent filter (invalid ID)
        let invalid_filter_id = Uuid::now_v7();
        let result = model.with_filter_select(invalid_filter_id);

        // Verify error
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should produce an error"),
            format!("Filter with ID {} not found.", invalid_filter_id)
        );
    }

    #[test]
    fn test_get_new_selection_no_selected_task() {
        // Setup a model with filtered tasks and no selected task
        let model = setup_model_with_tasks();

        // Use `get_new_selection` to get the first task when no task is selected
        let filtered_tasks = model.filtered_tasks.clone();
        let new_selection = model.get_new_selection(&filtered_tasks, None);

        // Verify the first task is selected
        assert_eq!(
            new_selection.unwrap(),
            *model.filtered_tasks.get_key_at_index(0).unwrap()
        );
    }

    #[test]
    fn test_get_new_selection_with_desired_task() {
        // Setup a model with filtered tasks
        let model = setup_model_with_tasks();

        // Select a specific task to be the new selection
        let filtered_tasks = model.filtered_tasks.clone();
        let desired_task_id = *model.filtered_tasks.get_key_at_index(1).unwrap();
        let new_selection = model.get_new_selection(&filtered_tasks, Some(desired_task_id));

        // Verify the specified task is selected
        assert_eq!(new_selection.unwrap(), desired_task_id);
    }

    #[test]
    fn test_get_new_selection_closest_task() {
        // Setup a model with filtered tasks
        let model = setup_model_with_tasks();

        // Select a task that isn't in filtered tasks and get the closest
        let mut filtered_tasks = model.filtered_tasks.clone();
        filtered_tasks = filtered_tasks.remove(&model.selected_task.unwrap_or_default());

        let new_selection = model.get_new_selection(&filtered_tasks, None);

        // Verify the closest task is selected
        assert_eq!(
            new_selection.unwrap(),
            *filtered_tasks.get_key_at_index(0).unwrap()
        );
    }

    #[test]
    fn test_with_tasks_updates_filtered_tasks_and_selection() {
        // Setup a model with tasks
        let model = setup_model_with_tasks();

        // Add a new task
        let new_task = Task::new("New Task");
        let mut tasks = model.tasks.clone();
        tasks = tasks.insert(*new_task.id, new_task.clone());

        // Update model with new tasks and filter selection
        let model = model.with_tasks(tasks.clone(), Some(*new_task.id));

        // Verify tasks, filtered tasks, and selection are updated
        assert_eq!(model.tasks.len(), 5); // Including the new task
        assert!(model.filtered_tasks.contains_key(&new_task.id));
        assert_eq!(model.selected_task.unwrap(), *new_task.id);
    }

    #[test]
    fn test_with_success_message() {
        let model = Model::new();
        let message = "Operation successful";
        let updated_model = model.with_success(message);

        assert_eq!(
            updated_model.message,
            DisplayMessage::Success(message.to_string())
        );
    }

    #[test]
    fn test_with_error_message() {
        let model = Model::new();
        let message = "An error occurred";
        let updated_model = model.with_error(message);

        assert_eq!(
            updated_model.message,
            DisplayMessage::Error(message.to_string())
        );
    }

    #[test]
    fn test_get_path_existing_selection() {
        let mut model = setup_model_with_tasks();
        let selected_task_id = *model.tasks.get_key_at_index(0).unwrap();
        model.selected_task = Some(selected_task_id);

        let path = model.get_path();
        assert!(path.is_some());
        assert_eq!(path.unwrap(), &vector![selected_task_id]);
    }

    #[test]
    fn test_get_path_no_selection() {
        let model = setup_model_with_tasks();
        let path = model.get_path();

        assert!(path.is_none());
    }

    #[test]
    fn test_with_tasks_preserves_filter() {
        let model = setup_model_with_tasks();

        // Add a new task
        let new_task = Task::new("New Task #new");
        let mut tasks = model.tasks.clone();
        tasks = tasks.insert(*new_task.id, new_task.clone());

        // Apply a filter that matches the new task
        let filter_condition = FilterCondition::new("#new").unwrap();
        let model = model.with_filter_condition(filter_condition);

        // Update model with new tasks
        let model = model.with_tasks(tasks, Some(*new_task.id));

        // Verify that the new task is in filtered tasks
        assert_eq!(model.filtered_tasks.len(), 1);
        assert!(model.filtered_tasks.contains_key(&new_task.id));
        assert_eq!(model.selected_task.unwrap(), *new_task.id);
    }
}
