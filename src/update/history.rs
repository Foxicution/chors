use crate::{model::Model, update::Message};
use std::collections::VecDeque;

pub struct History {
    pub undo_stack: VecDeque<(Model, Message)>,
    pub redo_stack: VecDeque<(Model, Message)>,
    pub last_action: Option<Message>,
    pub max_history: usize,
}

impl History {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            last_action: None,
            max_history,
        }
    }

    pub fn push(&mut self, model: &Model, message: &Message) {
        if self.undo_stack.len() == self.max_history {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back((model.clone(), message.clone()));
        self.redo_stack.clear();
        self.last_action = None;
    }

    pub fn undo(&mut self, current_model: &Model) -> Option<Model> {
        let (prev_model, prev_message) = self.undo_stack.pop_back()?;
        self.redo_stack
            .push_back((current_model.clone(), prev_message.clone()));
        self.last_action = Some(prev_message);
        Some(prev_model)
    }

    pub fn redo(&mut self, current_model: &Model) -> Option<Model> {
        let (next_model, next_message) = self.redo_stack.pop_back()?;
        self.undo_stack
            .push_back((current_model.clone(), next_message.clone()));
        self.last_action = Some(next_message);
        Some(next_model)
    }

    pub fn last_action(&self) -> Option<&Message> {
        self.last_action.as_ref()
    }
}
