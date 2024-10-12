mod model;
mod update;

use model::filter::{Filter, FilterCondition};
use model::model::Model;
use model::task::Task;
use update::{message::Message, update::update};

fn main() {
    let mut model = Model::new();

    // Create some tasks
    let mut task1 = Task::new("Complete the urgent report #work @office".to_string());
    let task2 = Task::new("Plan the birthday party #personal @home".to_string());
    let mut task3 = Task::new("Buy groceries #errand @shopping".to_string());
    let task4 = Task::new("Read a book #personal @leisure".to_string());
    let task5 = Task::new("Finish the project presentation #work @project".to_string());
    let task6 = Task::new("Urgent repair needed #urgent @home".to_string());

    // Mark some tasks as completed
    task1.mark_completed();
    task3.mark_completed();

    // Add tasks to the model using the update function
    update(
        Message::AddTask {
            task: task1,
            path: &[],
        },
        &mut model,
    );
    update(
        Message::AddTask {
            task: task2,
            path: &[],
        },
        &mut model,
    );
    update(
        Message::AddTask {
            task: task3,
            path: &[],
        },
        &mut model,
    );
    update(
        Message::AddTask {
            task: task4,
            path: &[],
        },
        &mut model,
    );
    update(
        Message::AddTask {
            task: task5,
            path: &[],
        },
        &mut model,
    );
    update(
        Message::AddTask {
            task: task6,
            path: &[],
        },
        &mut model,
    );

    // Define and add a filter
    let filter_expr = "not [x] and (\"project\" or @home)";
    let new_filter = Filter::new(
        "Active Project or Home Tasks".to_string(),
        FilterCondition::new(filter_expr.to_string()).unwrap(),
    );
    let filter_id = new_filter.id;

    // Add filter to the model using the update function
    update(Message::AddFilter { filter: new_filter }, &mut model);

    // Select the filter by sending the message
    update(
        Message::SelectFilter {
            filter_id: &filter_id,
        },
        &mut model,
    );

    // Display the applied filter
    println!("Filter Expression: {}", model.current_filter.expression);
    println!("All Tasks:");
    for task in model.tasks.values() {
        println!("{}", task);
    }

    // Print the filtered tasks after applying the filter
    println!("\nFiltered Tasks:");
    for path in model.filtered_tasks.values() {
        println!("{}", model.get_task(path).unwrap());
    }
}
