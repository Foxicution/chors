use crate::model::{Field, Filter, FilterCondition, Mode, Overlay, Task};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    // Task management
    AddSiblingTask(Task),
    AddChildTask(Task),
    RemoveTask(Vec<Uuid>),
    FlipCompleted(Vec<Uuid>),

    // Filter management
    AddFilter(Filter),
    SelectFilter(Uuid),
    ApplyFilter(String),

    // Navigation
    Navigate(Direction),

    // Modes
    SetMode(Mode),
    SetOverlay(Overlay),

    // Input
    SetInput(Field),
    // AddChar(char),
    // PopChar,
    // PopWord,
    // JumpWord(Direction),
    // Move(Direction),
    // JumpStart,
    // JumpEnd,

    // History
    Undo,
    Redo,

    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
}
