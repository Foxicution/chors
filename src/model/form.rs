use crate::update::Direction;
use rpds::HashTrieMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub cursor: usize,
    pub text: String,
}

impl Field {
    pub fn new() -> Self {
        Field {
            cursor: 0,
            text: String::new(),
        }
    }

    pub fn with_cursor_jump_word(&self, direction: &Direction) -> Self {
        let is_boundary = |c: char| c.is_whitespace() || c == '@' || c == '#';

        let new_cursor = match direction {
            Direction::Up => {
                let trimmed_text = &self.text[..self.cursor];

                if let Some(start_of_word) = trimmed_text.rfind(|c| !is_boundary(c)) {
                    // Check if the cursor is already at the start of the current word
                    if self.cursor > start_of_word + 1 {
                        trimmed_text[..start_of_word + 1]
                            .rfind(is_boundary)
                            .map_or(0, |i| i + 1)
                    } else {
                        // Move to the start of the previous word
                        trimmed_text[..start_of_word]
                            .rfind(is_boundary)
                            .map_or(0, |i| i + 1)
                    }
                } else {
                    0 // No previous word boundary, go to the start
                }
            }
            Direction::Down => {
                let remaining_text = &self.text[self.cursor..];

                if let Some(end_of_word) = remaining_text.find(|c| !is_boundary(c)) {
                    // Move right to the end of the current word or to the next word's start
                    let after_word_boundary = end_of_word
                        + remaining_text[end_of_word..]
                            .find(is_boundary)
                            .unwrap_or(remaining_text.len());

                    self.cursor + after_word_boundary
                } else {
                    self.text.len() // No next word boundary, go to the end
                }
            }
        };

        Self {
            cursor: new_cursor.max(0).min(self.text.len()),
            ..self.clone()
        }
    }

    pub fn with_popped_char(&self) -> Self {
        if self.cursor == 0 || self.text.is_empty() {
            return self.clone();
        }
        let mut text = self.text.clone();
        text.remove(self.cursor - 1);
        Self {
            text,
            cursor: self.cursor - 1,
            ..self.clone()
        }
    }

    pub fn with_popped_word(&self) -> Self {
        if self.cursor == 0 || self.text.is_empty() {
            return self.clone(); // If at the beginning or empty, no-op
        }
        let mut text = self.text[..self.cursor].to_string();
        let trimmed_text = text.trim_end();
        let last_space = trimmed_text.rfind(' ').unwrap_or(0);
        text.truncate(last_space);
        text.push_str(&self.text[self.cursor..]); // Preserve the text after the cursor
        Self {
            text,
            cursor: last_space,
            ..self.clone()
        }
    }

    pub fn with_inserted_char(&self, ch: char) -> Self {
        let mut text = self.text.clone();
        text.insert(self.cursor, ch);

        Self {
            text,
            cursor: self.cursor + 1,
            ..self.clone()
        }
    }

    pub fn with_cursor_move(&self, direction: &Direction) -> Self {
        let cursor = match direction {
            Direction::Down => (self.cursor + 1).min(self.text.len()),
            Direction::Up => self.cursor.saturating_sub(1),
        };

        Self {
            cursor,
            ..self.clone()
        }
    }

    pub fn with_cursor(&self, position: usize) -> Self {
        Self {
            cursor: position.min(self.text.len()),
            ..self.clone()
        }
    }

    pub fn with_text(&self, text: String) -> Self {
        Self {
            text,
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Form {
    active: String,
    fields: HashTrieMap<String, Field>,
}
