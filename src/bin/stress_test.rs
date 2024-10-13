use chors::model::filter::{Filter, FilterCondition};
use chors::model::model::Model;
use chors::model::task::Task;
use chors::update::{message::Message, update::update};
use std::time::Instant;
use uuid::Uuid;

// Helper function to create a task with a unique description
fn create_task_with_description(description: &str) -> Task {
    Task::new(description.to_string())
}

// Stress test function
fn main() {
    let mut model = Model::new();

    // Start the timer
    let start = Instant::now();

    // 1. Add a large number of tasks
    let num_tasks = 1_000; // Adjust this number based on your testing needs
    for i in 0..num_tasks {
        let task_description = format!("Task number {}", i);
        let task = create_task_with_description(&task_description);
        model = update(Message::AddSiblingTask { task }, &model);
    }

    let duration = start.elapsed();
    println!("Added {} tasks in: {:?}", num_tasks, duration);

    // 2. Add a large number of subtasks to the first task
    let mut parent_task_id = Uuid::now_v7(); // Replace with a real task ID after adding a task
    if let Some((task_id, _)) = model.tasks.iter().next() {
        parent_task_id = *task_id;
    }

    let subtask_start = Instant::now();
    let num_subtasks = 1_000;
    for i in 0..num_subtasks {
        let subtask_description = format!("Subtask number {}", i);
        let subtask = create_task_with_description(&subtask_description);
        model = update(Message::AddChildTask { task: subtask }, &model);
    }

    let subtask_duration = subtask_start.elapsed();
    println!(
        "Added {} subtasks to task {} in: {:?}",
        num_subtasks, parent_task_id, subtask_duration
    );

    // 3. Apply a filter
    let filter_start = Instant::now();
    let filter_expr = "\"Task\" or \"Subtask\"";
    let filter = Filter::new(
        "Task Filter".to_string(),
        FilterCondition::new(filter_expr.to_string()).unwrap(),
    );
    let filter_id = filter.id;

    model = update(Message::AddFilter { filter }, &model);
    model = update(Message::SelectFilter { filter_id }, &model);

    let filter_duration = filter_start.elapsed();
    println!("Applied filter on tasks in: {:?}", filter_duration);

    // 4. Reorder tasks
    // let reorder_start = Instant::now();
    // for i in 0..num_tasks / 2 {
    //     model = update(
    //         Message::Move {
    //             direction: Direction::Down,
    //         },
    //         &model,
    //     );
    // }

    // let reorder_duration = reorder_start.elapsed();
    // println!("Reordered tasks in: {:?}", reorder_duration);

    // Total time
    let total_duration = start.elapsed();
    println!("Total time for stress test: {:?}", total_duration);
}
