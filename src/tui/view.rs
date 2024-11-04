use crate::model::Model;
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::io::Stdout;

type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn ui(frame: &mut Frame, model: &Model) {
    let size = frame.size();
    let available_height = size.height.saturating_sub(2);
}
