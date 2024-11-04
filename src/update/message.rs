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
            (Message::Undo, Message::Undo) => true,
            (Message::Redo, Message::Redo) => true,
            (Message::AddSiblingTask { task: t1 }, Message::AddSiblingTask { task: t2 }) => {
                t1 == t2
            }
            (Message::AddChildTask { task: t1 }, Message::AddChildTask { task: t2 }) => t1 == t2,
            (Message::RemoveTask { path: p1 }, Message::RemoveTask { path: p2 }) => p1 == p2,
            (Message::AddFilter { filter: f1 }, Message::AddFilter { filter: f2 }) => f1 == f2,
            (
                Message::SelectFilter { filter_id: id1 },
                Message::SelectFilter { filter_id: id2 },
            ) => id1 == id2,
            (Message::ApplyFilter { filter: f1 }, Message::ApplyFilter { filter: f2 }) => f1 == f2,
            (Message::Navigate { direction: d1 }, Message::Navigate { direction: d2 }) => d1 == d2,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
}
