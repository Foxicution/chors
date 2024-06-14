use indexmap::IndexMap;
use ratatui::widgets::ListState;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub completed: bool,
    pub subtasks: IndexMap<Uuid, Task>,
}

impl Task {
    pub fn new(description: &str) -> Self {
        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: description.to_string(),
            completed: false,
            subtasks: IndexMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppMode {
    Normal,
    AddingTask,
    AddingSubtask,
}

pub struct AppState {
    pub tasks: IndexMap<Uuid, Task>,
    pub list_state: ListState,
    pub mode: AppMode,
    pub input: String,
    pub nav: IndexMap<Uuid, Vec<Uuid>>,
    pub selected: Option<Uuid>,
}

impl AppState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(None);
        Self {
            tasks: IndexMap::new(),
            list_state,
            mode: AppMode::Normal,
            input: String::new(),
            nav: IndexMap::new(),
            selected: None,
        }
    }

    // Needed for later
    // fn build_nav_list(&self, tasks: &[Task]) -> Vec<Vec<usize>> {
    //     let mut nav = Vec::new();
    //     let mut path = Vec::new();
    //     self.collect_nav_list(&tasks, &mut path, &mut nav);
    //     nav
    // }

    // fn collect_nav_list(&self, tasks: &[Task], path: &mut Vec<usize>, nav: &mut Vec<Vec<usize>>) {
    //     for (i, task) in tasks.iter().enumerate() {
    //         let mut current_path = path.clone();
    //         current_path.push(i);
    //         nav.push(current_path.clone());

    //         if !task.subtasks.is_empty() {
    //             self.collect_nav_list(&task.subtasks, &mut current_path, nav);
    //         }
    //     }
    // }

    // TODO: Refactor repeating logic
    pub fn get_task_list(&self) -> &IndexMap<Uuid, Task> {
        if let Some(selected) = self.selected {
            let nav_path = &self.nav.get(&selected).unwrap();
            let mut current_tasks = &self.tasks;
            for &uuid in &nav_path[..nav_path.len() - 1] {
                // Extract the immutable reference to subtasks
                if current_tasks.get(&uuid).is_none() {
                    return current_tasks;
                }
                current_tasks = &current_tasks[&uuid].subtasks;
            }
            current_tasks
        } else {
            &self.tasks
        }
    }

    pub fn get_task_list_mut(&mut self) -> &mut IndexMap<Uuid, Task> {
        if let Some(selected) = self.list_state.selected() {
            let nav_path = &self.nav[selected];
            let mut current_tasks = &mut self.tasks;
            for &uuid in &nav_path[..nav_path.len() - 1] {
                // Extract the mutable reference to subtasks without keeping the mutable borrow active
                if current_tasks.get_mut(&uuid).is_none() {
                    return current_tasks;
                }
                current_tasks = &mut current_tasks[&uuid].subtasks;
            }
            current_tasks
        } else {
            &mut self.tasks
        }
    }

    pub fn get_task(&self) -> Option<&Task> {
        if let Some(selected) = self.list_state.selected() {
            let nav_path = &self.nav[selected];
            let mut current_tasks = &self.tasks;
            for &uuid in &nav_path[..nav_path.len() - 1] {
                if let Some(t) = current_tasks.get(&uuid) {
                    current_tasks = &t.subtasks;
                } else {
                    return None;
                }
            }
            current_tasks.get(nav_path.last().unwrap())
        } else {
            None
        }
    }

    pub fn get_task_mut(&mut self) -> Option<&mut Task> {
        if let Some(selected) = self.list_state.selected() {
            let nav_path = &self.nav[selected];
            let mut current_tasks = &mut self.tasks;
            for &uuid in &nav_path[..nav_path.len() - 1] {
                if let Some(t) = current_tasks.get_mut(&uuid) {
                    current_tasks = &mut t.subtasks;
                } else {
                    return None;
                }
            }
            current_tasks.get_mut(nav_path.last().unwrap())
        } else {
            None
        }
    }

    pub fn add_task(&mut self) {
        let new_task = Task::new(&self.input);
        let new_id = new_task.id;
        self.get_task_list_mut().insert(new_task.id, new_task);
        self.selected = Some(new_id);
    }

    pub fn add_subtask(&mut self) {
        let new_task = Task::new(&self.input);
        let new_id = new_task.id;
        if let Some(task) = self.get_task_mut() {
            task.subtasks.insert(new_task.id, new_task);
            self.selected = Some(new_id);
        } else {
            todo!("Implement a message that subtask can't be added if there is no task selected!")
        }
    }
}
