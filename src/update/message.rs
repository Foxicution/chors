use uuid::Uuid;

use crate::model::{
    filter::{Condition, Filter, FilterCondition},
    task::Task,
};

pub enum Message<'a> {
    // Task management
    AddTask {
        task: Task,
        path: &'a [Uuid],
    },
    RemoveTask {
        path: &'a [Uuid],
    },
    UpdateTask {
        task: &'a mut Task,
        update: TaskUpdate,
    },
    MoveTask {
        old_path: &'a [Uuid],
        new_path: &'a [Uuid],
    },

    // Filter management
    AddFilter {
        filter: Filter,
    },
    SelectFilter {
        filter_id: &'a Uuid,
    },
    UpdateFilter {
        filter: &'a mut Filter,
        update: FilterUpdate,
    },
    UpdateCurrentFilter {
        expression: String,
    },
    ApplyFilterCondition,

    // Navigation
    Move {
        direction: Direction,
    },
    Select {
        index: usize,
    },

    // Erorr
    ErrorOccured {
        message: &'a str,
    },
}

pub struct TaskUpdate {
    pub description: Option<String>,
    pub completed: Option<bool>,
}

pub struct FilterUpdate {
    pub name: Option<String>,
    pub filter_condition: Option<FilterCondition>,
}

pub enum Direction {
    Up,
    Down,
}
