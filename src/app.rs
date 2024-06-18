use indexmap::IndexMap;
use ratatui::widgets::ListState;
use std::collections::HashSet;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub completed: bool,
    pub subtasks: IndexMap<Uuid, Task>,
    pub tags: HashSet<String>,
    pub contexts: HashSet<String>,
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

    pub fn update_description(&mut self, new_description: &str) {
        self.description = new_description.to_string();
        self.tags.clear();
        self.contexts.clear();
        self.extract_tags_and_contexts();
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum AppMode {
    Normal,
    AddingTask,
    AddingSubtask,
    DebugOverlay,
    AddingFilterCriterion,
    ViewMode,
    Quit,
}

#[derive(Debug)]
pub struct AppState {
    pub tasks: IndexMap<Uuid, Task>,
    pub list_state: ListState,
    pub mode: AppMode,
    pub input: String,
    pub nav: IndexMap<Uuid, Vec<Uuid>>,
    pub selected: Option<Uuid>,
    pub tags: HashSet<String>,
    pub contexts: HashSet<String>,
    pub autocomplete_suggestions: Vec<String>,
    pub debug_scroll: u16,
    pub current_view: View,
    pub saved_views: IndexMap<String, View>,
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
            tags: HashSet::new(),
            contexts: HashSet::new(),
            autocomplete_suggestions: Vec::new(),
            debug_scroll: 0,
            current_view: View {
                filter_lists: Vec::new(),
            },
            saved_views: IndexMap::new(),
        }
    }

    pub fn save_current_view_as_view(&mut self, name: &str) {
        self.saved_views
            .insert(name.to_string(), self.current_view.clone());
    }

    pub fn load_view(&mut self, name: &str) {
        if let Some(view) = self.saved_views.get(name) {
            self.current_view = view.clone();
        }
    }

    pub fn add_filter_list(&mut self, filter_list: FilterList) {
        self.current_view.filter_lists.push(filter_list);
    }

    fn get_path(&self) -> Vec<Uuid> {
        if let Some(selected) = self.selected {
            self.nav[&selected].clone()
        } else {
            vec![]
        }
    }

    fn get_task_list(&self, path: &[Uuid]) -> &IndexMap<Uuid, Task> {
        let mut current_tasks = &self.tasks;
        for &uuid in &path[..path.len().saturating_sub(1)] {
            current_tasks = &current_tasks[&uuid].subtasks;
        }
        current_tasks
    }

    fn get_task_list_mut(&mut self, path: &[Uuid]) -> &mut IndexMap<Uuid, Task> {
        let mut current_tasks = &mut self.tasks;
        for &uuid in &path[..path.len().saturating_sub(1)] {
            current_tasks = &mut current_tasks[&uuid].subtasks;
        }
        current_tasks
    }

    fn get_task(&self, path: &[Uuid]) -> Option<&Task> {
        self.get_task_list(path).get(
            path.last()
                .expect("Path length should always be >0 when looking for a task."),
        )
    }

    fn get_task_mut(&mut self, path: &[Uuid]) -> Option<&mut Task> {
        self.get_task_list_mut(path).get_mut(
            path.last()
                .expect("Path length should always be >0 when looking for a task."),
        )
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

    pub fn switch_mode(&mut self, mode: AppMode) {
        self.mode = mode;
        self.input.clear();
    }

    pub fn toggle_task_completion(&mut self) {
        let path = self.get_path();
        if let Some(task) = self.get_task_mut(&path) {
            task.completed = !task.completed;
        }
    }
}
