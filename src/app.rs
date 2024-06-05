use ratatui::widgets::ListState;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub completed: bool,
}

impl Task {
    pub fn new(description: &str) -> Self {
        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: description.to_string(),
            completed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppMode {
    Normal,
    AddingTask,
}

pub struct AppState {
    pub tasks: Vec<Task>,
    pub list_state: ListState,
    pub mode: AppMode,
    pub input: String,
}

impl AppState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            tasks: Vec::new(),
            list_state,
            mode: AppMode::Normal,
            input: String::new(),
        }
    }
}
