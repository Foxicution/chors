use uuid::Uuid;

use crate::model::{Filter, FilterCondition, Mode, Overlay, Task};
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
    SetMode(Mode),
    SetOverlay(Overlay),

    // Input
    SetInput(String),

    // History
    Undo,
    Redo,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Task management
            (Message::AddSiblingTask(t1), Message::AddSiblingTask(t2)) => t1 == t2,
            (Message::AddChildTask(t1), Message::AddChildTask(t2)) => t1 == t2,
            (Message::RemoveTask(p1), Message::RemoveTask(p2)) => p1 == p2,
            (Message::FlipCompleted(p1), Message::FlipCompleted(p2)) => p1 == p2,

            // Filter management
            (Message::AddFilter(f1), Message::AddFilter(f2)) => f1 == f2,
            (Message::SelectFilter(id1), Message::SelectFilter(id2)) => id1 == id2,
            (Message::ApplyFilter(f1), Message::ApplyFilter(f2)) => f1 == f2,

            // Navigation
            (Message::Navigate(d1), Message::Navigate(d2)) => d1 == d2,

            // Mode
            (Message::SetMode(m1), Message::SetMode(m2)) => m1 == m2,
            (Message::SetOverlay(o1), Message::SetOverlay(o2)) => o1 == o2,

            // Input
            (Message::SetInput(i1), Message::SetInput(i2)) => i1 == i2,

            // History
            (Message::Undo, Message::Undo) => true,
            (Message::Redo, Message::Redo) => true,

            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
}
