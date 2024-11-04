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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Model;
    use crate::update::message::Message;

    #[test]
    fn test_push_and_undo_redo() {
        let mut history = History::new(2); // Small max_history to test limit
        let model1 = Model::new();
        let model2 = model1.clone();
        let message1 = Message::Undo;

        // Push first state
        history.push(&model1, &message1);
        assert_eq!(history.undo_stack.len(), 1);

        // Push second state
        history.push(&model2, &message1);
        assert_eq!(history.undo_stack.len(), 2);

        // Push third state, should pop the first one due to max_history limit
        history.push(&model1, &message1);
        assert_eq!(history.undo_stack.len(), 2);

        // Undo twice
        let _ = history.undo(&model1);
        let _ = history.undo(&model1);

        assert_eq!(history.redo_stack.len(), 2);

        // Redo once
        let _ = history.redo(&model1);
        assert_eq!(history.redo_stack.len(), 1);
    }

    #[test]
    fn test_last_action() {
        let mut history = History::new(100);
        let model = Model::new();
        let message = Message::Undo;

        history.push(&model, &message);

        // Undo an action
        let _ = history.undo(&model);
        assert_eq!(history.last_action(), Some(&Message::Undo));

        // Redo an action
        let _ = history.redo(&model);
        assert_eq!(history.last_action(), Some(&Message::Undo));
    }
}
