use uuid::Uuid;

use crate::model::{Filter, FilterCondition, Task};

#[derive(Debug, Clone)]
pub enum Message {
    // Task management
    AddSiblingTask { task: Task },
    AddChildTask { task: Task },
    RemoveTask { path: Vec<Uuid> },

    // Filter management
    AddFilter { filter: Filter },
    SelectFilter { filter_id: Uuid },
    ApplyFilter { filter: FilterCondition },

    // Navigation
    Navigate { direction: Direction },

    // History
    Undo,
    Redo,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Message::AddSiblingTask { task: t1 }, Message::AddSiblingTask { task: t2 }) => {
                t1.id == t2.id
            }
            (Message::AddChildTask { task: t1 }, Message::AddChildTask { task: t2 }) => {
                t1.id == t2.id
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
}
