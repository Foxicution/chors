use rpds::HashTrieMap;
use std::collections::HashMap as StdHashMap;
use std::time::Instant;
use uuid::{NoContext, Timestamp, Uuid};

const NUM_TASKS: usize = 100_000;

#[derive(Clone)]
pub struct ImmutableModel {
    pub tasks: HashTrieMap<Uuid, Task>, // Using rpds::HashTrieMap for persistent map
    pub filters: HashTrieMap<Uuid, Filter>,
    pub selected_filter_id: Option<Uuid>,
    pub current_filter: FilterCondition,
    pub filtered_tasks: HashTrieMap<Uuid, Vec<Uuid>>,
    pub selected_task: Option<Uuid>,
    pub error: Option<String>,
}

impl ImmutableModel {
    pub fn new() -> Self {
        let empty_filter_condition = FilterCondition::new(String::new()).unwrap();
        let default_filter = Filter::new("default".to_string(), empty_filter_condition.clone());
        let default_filter_id = default_filter.id;

        ImmutableModel {
            tasks: HashTrieMap::new(),
            filters: HashTrieMap::new().insert(default_filter_id, default_filter),
            selected_filter_id: Some(default_filter_id),
            current_filter: empty_filter_condition,
            filtered_tasks: HashTrieMap::new(),
            selected_task: None,
            error: None,
        }
    }

    pub fn add_task(&self, task: Task) -> Self {
        let new_tasks = self.tasks.insert(task.id, task);
        ImmutableModel {
            tasks: new_tasks,
            ..self.clone()
        }
    }

    pub fn modify_task(&self, task_id: &Uuid, new_description: &str) -> Self {
        if let Some(mut task) = self.tasks.get(task_id).cloned() {
            task.description = new_description.to_string();
            let new_tasks = self.tasks.insert(task_id.clone(), task);
            ImmutableModel {
                tasks: new_tasks,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }
}

pub struct MutableModel {
    pub tasks: StdHashMap<Uuid, Task>, // Using standard HashMap for mutable model
    pub filters: StdHashMap<Uuid, Filter>,
    pub selected_filter_id: Option<Uuid>,
    pub current_filter: FilterCondition,
    pub filtered_tasks: StdHashMap<Uuid, Vec<Uuid>>,
    pub selected_task: Option<Uuid>,
    pub error: Option<String>,
}

impl MutableModel {
    pub fn new() -> Self {
        let empty_filter_condition = FilterCondition::new(String::new()).unwrap();
        let default_filter = Filter::new("default".to_string(), empty_filter_condition.clone());
        let default_filter_id = default_filter.id;

        MutableModel {
            tasks: StdHashMap::new(),
            filters: StdHashMap::from([(default_filter_id, default_filter)]),
            selected_filter_id: Some(default_filter_id),
            current_filter: empty_filter_condition,
            filtered_tasks: StdHashMap::new(),
            selected_task: None,
            error: None,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    pub fn modify_task(&mut self, task_id: &Uuid, new_description: &str) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.description = new_description.to_string();
        }
    }
}

#[derive(Clone)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub tags: Vec<String>,
    pub contexts: Vec<String>,
    pub completed: Option<bool>,
    pub subtasks: HashTrieMap<Uuid, Task>,
}

impl Task {
    pub fn new(description: &str) -> Self {
        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: description.to_string(),
            tags: vec![],
            contexts: vec![],
            completed: None,
            subtasks: HashTrieMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct Filter {
    pub id: Uuid,
    pub name: String,
    pub filter_condition: FilterCondition,
}

impl Filter {
    pub fn new(name: String, filter_condition: FilterCondition) -> Self {
        Filter {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            name,
            filter_condition,
        }
    }
}

#[derive(Clone)]
pub struct FilterCondition {
    pub expression: String,
}

impl FilterCondition {
    pub fn new(expression: String) -> Result<Self, String> {
        Ok(FilterCondition { expression })
    }
}

// Stress tests
fn stress_test() {
    // Mutable model
    let mut mutable_model = MutableModel::new();
    let mut task_ids = Vec::new();

    let start_mutable = Instant::now();
    for i in 0..NUM_TASKS {
        let task = Task::new(&format!("Task {}", i));
        task_ids.push(task.id);
        mutable_model.add_task(task);
    }
    let duration_mutable_add = start_mutable.elapsed();
    println!(
        "Mutable model: Added {} tasks in {:?}",
        NUM_TASKS, duration_mutable_add
    );

    let start_mutable_mod = Instant::now();
    for i in 0..NUM_TASKS {
        mutable_model.modify_task(&task_ids[i], &format!("Modified Task {}", i));
    }
    let duration_mutable_mod = start_mutable_mod.elapsed();
    println!(
        "Mutable model: Modified {} tasks in {:?}",
        NUM_TASKS, duration_mutable_mod
    );

    // Immutable model
    let mut immutable_model = ImmutableModel::new();
    let mut task_ids = Vec::new();

    let start_immutable = Instant::now();
    for i in 0..NUM_TASKS {
        let task = Task::new(&format!("Task {}", i));
        task_ids.push(task.id);
        immutable_model = immutable_model.add_task(task);
    }
    let duration_immutable_add = start_immutable.elapsed();
    println!(
        "Immutable model (rpds): Added {} tasks in {:?}",
        NUM_TASKS, duration_immutable_add
    );

    let start_immutable_mod = Instant::now();
    for i in 0..NUM_TASKS {
        immutable_model =
            immutable_model.modify_task(&task_ids[i], &format!("Modified Task {}", i));
    }
    let duration_immutable_mod = start_immutable_mod.elapsed();
    println!(
        "Immutable model (rpds): Modified {} tasks in {:?}",
        NUM_TASKS, duration_immutable_mod
    );
}

fn main() {
    stress_test();
}
