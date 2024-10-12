// mod model;
// mod update;

// use model::filter::{Filter, FilterCondition};
// use model::model::Model;
// use model::task::Task;
// use update::{message::Message, update::update};

// fn main() {
//     let mut model = Model::new();

//     // Create some tasks
//     let mut task1 = Task::new("Complete the urgent report #work @office".to_string());
//     let task2 = Task::new("Plan the birthday party #personal @home".to_string());
//     let mut task3 = Task::new("Buy groceries #errand @shopping".to_string());
//     let task4 = Task::new("Read a book #personal @leisure".to_string());
//     let task5 = Task::new("Finish the project presentation #work @project".to_string());
//     let task6 = Task::new("Urgent repair needed #urgent @home".to_string());

//     // Mark some tasks as completed
//     task1.mark_completed();
//     task3.mark_completed();

//     // Add tasks to the model using the update function
//     update(
//         Message::AddTask {
//             task: task1,
//             path: &[],
//         },
//         &mut model,
//     );
//     update(
//         Message::AddTask {
//             task: task2,
//             path: &[],
//         },
//         &mut model,
//     );
//     update(
//         Message::AddTask {
//             task: task3,
//             path: &[],
//         },
//         &mut model,
//     );
//     update(
//         Message::AddTask {
//             task: task4,
//             path: &[],
//         },
//         &mut model,
//     );
//     update(
//         Message::AddTask {
//             task: task5,
//             path: &[],
//         },
//         &mut model,
//     );
//     update(
//         Message::AddTask {
//             task: task6,
//             path: &[],
//         },
//         &mut model,
//     );

//     // Define and add a filter
//     let filter_expr = "not [x] and (\"project\" or @home)";
//     let new_filter = Filter::new(
//         "Active Project or Home Tasks".to_string(),
//         FilterCondition::new(filter_expr.to_string()).unwrap(),
//     );
//     let filter_id = new_filter.id;

//     // Add filter to the model using the update function
//     update(Message::AddFilter { filter: new_filter }, &mut model);

//     // Select the filter by sending the message
//     update(
//         Message::SelectFilter {
//             filter_id: &filter_id,
//         },
//         &mut model,
//     );

//     // Display the applied filter
//     println!("Filter Expression: {}", model.current_filter.expression);
//     println!("All Tasks:");
//     for task in model.tasks.values() {
//         println!("{}", task);
//     }

//     // Print the filtered tasks after applying the filter
//     println!("\nFiltered Tasks:");
//     for path in model.filtered_tasks.values() {
//         println!("{}", model.get_task(path).unwrap());
//     }
// }

// use indexmap::IndexMap;
// use std::rc::Rc;
// use std::time::Instant;
// use uuid::{NoContext, Timestamp, Uuid};

// const NUM_TASKS: usize = 100_000;

// #[derive(Clone)]
// pub struct Model {
//     pub tasks: IndexMap<Uuid, Task>,

//     pub filters: IndexMap<Uuid, Filter>,
//     pub selected_filter_id: Option<Uuid>,
//     pub current_filter: FilterCondition,

//     pub filtered_tasks: IndexMap<Uuid, Vec<Uuid>>,
//     pub selected_task: Option<Uuid>,
//     pub error: Option<String>,
// }

// impl Model {
//     pub fn new() -> Self {
//         let empty_filter_condition = FilterCondition::new(String::new()).unwrap();
//         let default_filter = Filter::new("default".to_string(), empty_filter_condition.clone());
//         let default_filter_id = default_filter.id;

//         Model {
//             tasks: IndexMap::new(),
//             filters: IndexMap::from([(default_filter_id, default_filter)]),
//             selected_filter_id: Some(default_filter_id),
//             current_filter: empty_filter_condition,
//             filtered_tasks: IndexMap::new(),
//             selected_task: None,
//             error: None,
//         }
//     }

//     pub fn add_task(&mut self, task: Task) {
//         self.tasks.insert(task.id, task);
//     }

//     pub fn modify_task(&mut self, task_id: &Uuid, new_description: &str) {
//         if let Some(task) = self.tasks.get_mut(task_id) {
//             task.description = new_description.to_string();
//         }
//     }

//     pub fn clone_and_add_task(&self, task: Task) -> Self {
//         let mut new_model = self.clone();
//         new_model.add_task(task);
//         new_model
//     }

//     pub fn clone_and_modify_task(&self, task_id: &Uuid, new_description: &str) -> Self {
//         let mut new_model = self.clone();
//         new_model.modify_task(task_id, new_description);
//         new_model
//     }
// }

// #[derive(Clone)]
// pub struct ImmutableModel {
//     pub tasks: Rc<IndexMap<Uuid, Task>>,

//     pub filters: Rc<IndexMap<Uuid, Filter>>,
//     pub selected_filter_id: Option<Uuid>,
//     pub current_filter: Rc<FilterCondition>,

//     pub filtered_tasks: Rc<IndexMap<Uuid, Vec<Uuid>>>,
//     pub selected_task: Option<Uuid>,
//     pub error: Option<String>,
// }

// impl ImmutableModel {
//     pub fn new() -> Self {
//         let empty_filter_condition = FilterCondition::new(String::new()).unwrap();
//         let default_filter = Filter::new("default".to_string(), empty_filter_condition.clone());
//         let default_filter_id = default_filter.id;

//         ImmutableModel {
//             tasks: Rc::new(IndexMap::new()),
//             filters: Rc::new(IndexMap::from([(default_filter_id, default_filter)])),
//             selected_filter_id: Some(default_filter_id),
//             current_filter: Rc::new(empty_filter_condition),
//             filtered_tasks: Rc::new(IndexMap::new()),
//             selected_task: None,
//             error: None,
//         }
//     }

//     pub fn add_task(&mut self, task: Task) -> Self {
//         let mut new_tasks = (*self.tasks).clone();
//         new_tasks.insert(task.id, task);

//         ImmutableModel {
//             tasks: Rc::new(new_tasks),
//             ..self.clone()
//         }
//     }

//     pub fn modify_task(&self, task_id: &Uuid, new_description: &str) -> Self {
//         let mut new_tasks = (*self.tasks).clone();
//         if let Some(task) = new_tasks.get_mut(task_id) {
//             task.description = new_description.to_string();
//         }
//         ImmutableModel {
//             tasks: Rc::new(new_tasks),
//             ..self.clone()
//         }
//     }
// }

// #[derive(Clone)]
// pub struct Task {
//     pub id: Uuid,
//     pub description: String,
//     pub tags: Vec<String>,
//     pub contexts: Vec<String>,
//     pub completed: Option<bool>,
//     pub subtasks: IndexMap<Uuid, Task>,
// }

// impl Task {
//     pub fn new(description: &str) -> Self {
//         Task {
//             id: Uuid::new_v7(Timestamp::now(NoContext)),
//             description: description.to_string(),
//             tags: vec![],
//             contexts: vec![],
//             completed: None,
//             subtasks: IndexMap::new(),
//         }
//     }
// }

// #[derive(Clone)]
// pub struct Filter {
//     pub id: Uuid,
//     pub name: String,
//     pub filter_condition: FilterCondition,
// }

// impl Filter {
//     pub fn new(name: String, filter_condition: FilterCondition) -> Self {
//         Filter {
//             id: Uuid::new_v7(Timestamp::now(NoContext)),
//             name,
//             filter_condition,
//         }
//     }
// }

// #[derive(Clone)]
// pub struct FilterCondition {
//     pub expression: String,
// }

// impl FilterCondition {
//     pub fn new(expression: String) -> Result<Self, String> {
//         Ok(FilterCondition { expression })
//     }
// }

// // Stress tests
// fn stress_test() {
//     let mut mutable_model = Model::new();
//     let mut task_ids = Vec::new();

//     // Stress test for mutable model
//     let start_mutable = Instant::now();
//     for i in 0..NUM_TASKS {
//         let task = Task::new(&format!("Task {}", i));
//         task_ids.push(task.id);
//         mutable_model.add_task(task);
//     }
//     let duration_mutable = start_mutable.elapsed();
//     println!(
//         "Mutable model: Added {} tasks in {:?}",
//         NUM_TASKS, duration_mutable
//     );

//     let start_mutable_mod = Instant::now();
//     for i in 0..NUM_TASKS {
//         mutable_model.modify_task(&task_ids[i], &format!("Modified Task {}", i));
//     }
//     let duration_mutable_mod = start_mutable_mod.elapsed();
//     println!(
//         "Mutable model: Modified {} tasks in {:?}",
//         NUM_TASKS, duration_mutable_mod
//     );

//     // Stress test for immutable model
//     let immutable_model = ImmutableModel::new();
//     let mut task_ids = Vec::new();
//     let start_immutable = Instant::now();
//     let mut current_model = immutable_model;
//     for i in 0..NUM_TASKS {
//         let task = Task::new(&format!("Task {}", i));
//         task_ids.push(task.id);
//         current_model = current_model.add_task(task);
//     }
//     let duration_immutable = start_immutable.elapsed();
//     println!(
//         "Immutable model: Added {} tasks in {:?}",
//         NUM_TASKS, duration_immutable
//     );

//     let start_immutable_mod = Instant::now();
//     for i in 0..NUM_TASKS {
//         current_model = current_model.modify_task(&task_ids[i], &format!("Modified Task {}", i));
//     }
//     let duration_immutable_mod = start_immutable_mod.elapsed();
//     println!(
//         "Immutable model: Modified {} tasks in {:?}",
//         NUM_TASKS, duration_immutable_mod
//     );
// }

// fn main() {
//     stress_test();
// }
// use im::HashMap;
// use indexmap::IndexMap;
// use std::rc::Rc;
// use std::time::Instant;
// use uuid::{NoContext, Timestamp, Uuid};

// const NUM_TASKS: usize = 100_000;

// #[derive(Clone)]
// pub struct Model {
//     pub tasks: HashMap<Uuid, Task>,
//     pub filters: HashMap<Uuid, Filter>,
//     pub selected_filter_id: Option<Uuid>,
//     pub current_filter: FilterCondition,
//     pub filtered_tasks: HashMap<Uuid, Vec<Uuid>>,
//     pub selected_task: Option<Uuid>,
//     pub error: Option<String>,
// }

// impl Model {
//     pub fn new() -> Self {
//         let empty_filter_condition = FilterCondition::new(String::new()).unwrap();
//         let default_filter = Filter::new("default".to_string(), empty_filter_condition.clone());
//         let default_filter_id = default_filter.id;

//         Model {
//             tasks: HashMap::new(),
//             filters: HashMap::unit(default_filter_id, default_filter),
//             selected_filter_id: Some(default_filter_id),
//             current_filter: empty_filter_condition,
//             filtered_tasks: HashMap::new(),
//             selected_task: None,
//             error: None,
//         }
//     }

//     pub fn add_task(&self, task: Task) -> Self {
//         let new_tasks = self.tasks.update(task.id, task);
//         Model {
//             tasks: new_tasks,
//             ..self.clone()
//         }
//     }

//     pub fn modify_task(&self, task_id: &Uuid, new_description: &str) -> Self {
//         let task_clone = self.tasks.get(task_id).unwrap().clone();
//         let new_tasks = self
//             .tasks
//             .update_with(task_id.clone(), task_clone, |_, mut task: Task| {
//                 task.description = new_description.to_string();
//                 task
//             });
//         Model {
//             tasks: new_tasks,
//             ..self.clone()
//         }
//     }
// }

// #[derive(Clone)]
// pub struct Task {
//     pub id: Uuid,
//     pub description: String,
//     pub tags: Vec<String>,
//     pub contexts: Vec<String>,
//     pub completed: Option<bool>,
//     pub subtasks: HashMap<Uuid, Task>,
// }

// impl Task {
//     pub fn new(description: &str) -> Self {
//         Task {
//             id: Uuid::new_v7(Timestamp::now(NoContext)),
//             description: description.to_string(),
//             tags: vec![],
//             contexts: vec![],
//             completed: None,
//             subtasks: HashMap::new(),
//         }
//     }
// }

// #[derive(Clone)]
// pub struct Filter {
//     pub id: Uuid,
//     pub name: String,
//     pub filter_condition: FilterCondition,
// }

// impl Filter {
//     pub fn new(name: String, filter_condition: FilterCondition) -> Self {
//         Filter {
//             id: Uuid::new_v7(Timestamp::now(NoContext)),
//             name,
//             filter_condition,
//         }
//     }
// }

// #[derive(Clone)]
// pub struct FilterCondition {
//     pub expression: String,
// }

// impl FilterCondition {
//     pub fn new(expression: String) -> Result<Self, String> {
//         Ok(FilterCondition { expression })
//     }
// }

// // Stress tests
// fn stress_test() {
//     let mut mutable_model = Model::new();
//     let mut task_ids = Vec::new();

//     // Stress test for mutable model
//     let start_mutable = Instant::now();
//     for i in 0..NUM_TASKS {
//         let task = Task::new(&format!("Task {}", i));
//         task_ids.push(task.id);
//         mutable_model = mutable_model.add_task(task);
//     }
//     let duration_mutable = start_mutable.elapsed();
//     println!(
//         "Persistent model: Added {} tasks in {:?}",
//         NUM_TASKS, duration_mutable
//     );

//     let start_mutable_mod = Instant::now();
//     for i in 0..NUM_TASKS {
//         mutable_model = mutable_model.modify_task(&task_ids[i], &format!("Modified Task {}", i));
//     }
//     let duration_mutable_mod = start_mutable_mod.elapsed();
//     println!(
//         "Persistent model: Modified {} tasks in {:?}",
//         NUM_TASKS, duration_mutable_mod
//     );
// }

// fn main() {
//     stress_test();
// }

use rpds::HashTrieMap;
use std::collections::HashMap as StdHashMap;
use std::time::Instant;
use uuid::{NoContext, Timestamp, Uuid};

const NUM_TASKS: usize = 1_000;

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
