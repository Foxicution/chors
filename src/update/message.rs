use uuid::Uuid;

use crate::model::{Filter, FilterCondition, Mode, Task};

#[derive(Debug, Clone)]
pub enum Message {
    // Task management
    AddSiblingTask(Task),
    AddChildTask(Task),
    RemoveTask(Vec<Uuid>),
    FlipCompleted(Vec<Uuid>),

    // Filter management
    AddFilter(Filter),
    SelectFilter(Uuid),
    ApplyFilter(FilterCondition),

    // Navigation
    Navigate(Direction),

    // Modes
    SwitchMode(Mode),

    // History
    Undo,
    Redo,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Message::Undo, Message::Undo) => true,
            (Message::Redo, Message::Redo) => true,
            (Message::FlipCompleted(p1), Message::FlipCompleted(p2)) => p1 == p2,
            (Message::AddSiblingTask(t1), Message::AddSiblingTask(t2)) => t1 == t2,
            (Message::AddChildTask(t1), Message::AddChildTask(t2)) => t1 == t2,
            (Message::RemoveTask(p1), Message::RemoveTask(p2)) => p1 == p2,
            (Message::AddFilter(f1), Message::AddFilter(f2)) => f1 == f2,
            (Message::SelectFilter(id1), Message::SelectFilter(id2)) => id1 == id2,
            (Message::ApplyFilter(f1), Message::ApplyFilter(f2)) => f1 == f2,
            (Message::Navigate(d1), Message::Navigate(d2)) => d1 == d2,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
}
