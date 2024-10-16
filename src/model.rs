use chrono::{DateTime, Local};
use indexmap::IndexMap;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub completed: bool,
    pub subtasks: IndexMap<Uuid, Task>,
    pub tags: HashSet<String>,
    pub contexts: HashSet<String>,
    pub start_time: Option<DateTime<Local>>,
    pub due_time: Option<DateTime<Local>>,
}

impl Task {
    pub fn new(description: &str) -> Self {
        let mut task = Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: description.to_string(),
            completed: false,
            subtasks: IndexMap::new(),
            tags: HashSet::new(),
            contexts: HashSet::new(),
            start_time: None,
            due_time: None,
        };
        task.extract_tags_and_contexts();
        task
    }

    fn extract_tags_and_contexts(&mut self) {
        for word in self.description.split_whitespace() {
            if word.starts_with('#') {
                self.tags.insert(word.to_string());
            } else if word.starts_with('@') {
                self.contexts.insert(word.to_string());
            }
        }
    }

    fn update_description(&mut self, new_description: &str) {
        self.description = new_description.to_string();
        self.tags.clear();
        self.contexts.clear();
        self.extract_tags_and_contexts();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
    Completed(bool),
    Tag(String),
    Context(String),
}

impl Filter {
    pub fn matches(&self, task: &Task) -> bool {
        match self {
            Filter::Completed(completed) => task.completed == *completed,
            Filter::Tag(tag) => task.tags.contains(tag),
            Filter::Context(context) => task.contexts.contains(context),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterList {
    pub filters: Vec<Filter>,
}

impl FilterList {
    pub fn matches(&self, task: &Task) -> bool {
        if self.filters.is_empty() {
            return true;
        }
        self.filters.iter().all(|filter| filter.matches(task))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub filter_lists: Vec<FilterList>,
}

impl View {
    pub fn matches(&self, task: &Task) -> bool {
        if self.filter_lists.is_empty() {
            return true;
        }
        self.filter_lists
            .iter()
            .any(|filter_list| filter_list.matches(task))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mode {
    List,
    Calendar,
    Quit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Overlay {
    None,
    AddingTask,
    AddingSubtask,
    AddingFilterCriterion,
    View,
    Navigation,
    Help,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub tasks: IndexMap<Uuid, Task>,
    #[serde(with = "list_state_serde")]
    pub list_state: ListState,
    pub mode: Mode,
    pub overlay: Overlay,
    pub input: String,
    pub command_input: String,
    pub taskbar_info: String,
    pub taskbar_message: String,
    pub nav: IndexMap<Uuid, Vec<Uuid>>,
    pub selected: Option<Uuid>,
    pub tags: HashSet<String>,
    pub contexts: HashSet<String>,
    pub autocomplete_suggestions: Vec<String>,
    pub debug_scroll: u16,
    pub current_view: View,
    pub selected_view: String,
    pub saved_views: IndexMap<String, View>,
    pub navigation_input: String,
}

impl Model {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(None);

        let current_view = View {
            filter_lists: Vec::new(),
        };
        let mut saved_views = IndexMap::new();
        let selected_view = "default".to_string();
        saved_views.insert(selected_view.clone(), current_view.clone());

        Self {
            tasks: IndexMap::new(),
            list_state,
            mode: Mode::List,
            overlay: Overlay::None,
            input: String::new(),
            command_input: String::new(),
            taskbar_info: String::new(),
            taskbar_message: String::new(),
            nav: IndexMap::new(),
            selected: None,
            tags: HashSet::new(),
            contexts: HashSet::new(),
            autocomplete_suggestions: Vec::new(),
            debug_scroll: 0,
            current_view,
            selected_view,
            saved_views,
            navigation_input: String::new(),
        }
    }

    pub fn set_taskbar_message(&mut self, message: &str) {
        self.taskbar_message = message.to_string();
    }

    pub fn clear_taskbar_message(&mut self) {
        self.taskbar_message.clear();
    }

    pub fn get_path(&self) -> Vec<Uuid> {
        match self.selected {
            Some(selected) => self.nav[&selected].clone(),
            None => vec![],
        }
    }

    fn get_task_list(&self, path: &[Uuid]) -> &IndexMap<Uuid, Task> {
        let mut current_tasks = &self.tasks;
        for &uuid in &path[..path.len().saturating_sub(1)] {
            current_tasks = &current_tasks[&uuid].subtasks;
        }
        current_tasks
    }

    pub fn get_task_list_mut(&mut self, path: &[Uuid]) -> &mut IndexMap<Uuid, Task> {
        let mut current_tasks = &mut self.tasks;
        for &uuid in &path[..path.len().saturating_sub(1)] {
            current_tasks = &mut current_tasks[&uuid].subtasks;
        }
        current_tasks
    }

    fn get_task(&self, path: &[Uuid]) -> Option<&Task> {
        match path.last() {
            Some(last) => self.get_task_list(path).get(last),
            None => None,
        }
    }

    pub fn get_task_mut(&mut self, path: &[Uuid]) -> Option<&mut Task> {
        match path.last() {
            Some(last) => self.get_task_list_mut(path).get_mut(last),
            None => None,
        }
    }

    pub fn add_task(&mut self) {
        let new_task = Task::new(&self.input);
        let new_id = new_task.id;
        let path = self.get_path();
        self.get_task_list_mut(&path).insert(new_task.id, new_task);
        self.selected = Some(new_id);
    }

    pub fn add_subtask(&mut self) {
        let new_task = Task::new(&self.input);
        let new_id = new_task.id;
        let path = self.get_path();
        if let Some(task) = self.get_task_mut(&path) {
            task.subtasks.insert(new_task.id, new_task);
            self.selected = Some(new_id);
        } else {
            todo!("Implement a message that subtask can't be added if there is no task selected!")
        }
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub enum Msg {
    NoOp,
    Quit,
    PushChar(char),
    PopChar,
    AddTask,
    AddSubtask,
    ToggleTaskCompletion,
    SwitchMode(Mode),
    SetOverlay(Overlay),
    NavigateTasks(Direction),
    ScrollDebug(Direction),
    HandleNavigation,
    JumpToEnd,
    AddFilterCriterion,
    SaveCurrentView(String),
    LoadView(String),
}

mod list_state_serde {
    use super::ListState;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct SerializableListState {
        selected: Option<usize>,
    }

    impl From<ListState> for SerializableListState {
        fn from(state: ListState) -> Self {
            Self {
                selected: state.selected(),
            }
        }
    }

    impl From<SerializableListState> for ListState {
        fn from(state: SerializableListState) -> Self {
            let mut list_state = ListState::default();
            list_state.select(state.selected);
            list_state
        }
    }

    pub fn serialize<S>(state: &ListState, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serializable_state = SerializableListState::from(state.clone());
        serializable_state.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ListState, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serializable_state = SerializableListState::deserialize(deserializer)?;
        Ok(ListState::from(serializable_state))
    }
}
