use crate::model::{
    filter::{Condition, Filter, FilterCondition},
    task::Task,
};
use std::rc::Rc;
use uuid::Uuid;

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
        task_id: Uuid,
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
        filter_id: Uuid,
        update: FilterUpdate<'a>,
    },
    UpdateCurrentFilter {
        expression: &'a str,
    },
    ApplyFilterCondition,

    // Navigation
    Move {
        direction: Direction,
    },
    Select {
        index: usize,
    },

    // Error
    ErrorOccured {
        message: &'a str,
    },
}

pub struct TaskUpdate {
    pub description: Option<String>,
    pub completed: Option<bool>,
}

pub struct FilterUpdate<'a> {
    pub name: Option<&'a str>,
    pub filter_condition: Option<Rc<FilterCondition>>,
}

pub enum Direction {
    Up,
    Down,
}
