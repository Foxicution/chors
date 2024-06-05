#![allow(unused)]

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use indexmap::IndexMap;
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io::{stdout, Result};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone)]
struct Task {
    id: Uuid,
    description: String,
    completed: bool,
}

impl Task {
    fn new(description: &str) -> Self {
        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: description.to_string(),
            completed: false,
        }
    }
}

fn main() -> Result<()> {
    // Terminal intialization
    stdout().execute(EnterAlternateScreen);
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Main loop
    let mut tasks: Vec<Task> = Vec::new();
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    loop {
        // Drawing the UI
        terminal.draw(|frame| {
            let size = frame.size();
            let items: Vec<ListItem> = tasks
                .iter()
                .map(|task| {
                    let status = if task.completed { "[x]" } else { "[ ]" };
                    ListItem::new(format!("{} {}", status, task.description))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Tasks"))
                .highlight_style(Style::default())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .bg(Color::LightBlue);

            frame.render_stateful_widget(list, size, &mut list_state)
        })?;

        // Accepting inputs
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('n') => {
                            tasks.push(Task::new("New Task"));
                            list_state.select(Some(tasks.len() - 1));
                        }
                        KeyCode::Char('c') => {
                            if let Some(selected) = list_state.selected() {
                                if let Some(task) = tasks.get_mut(selected) {
                                    task.completed = !task.completed
                                }
                            }
                        }
                        KeyCode::Char('k') => {
                            if let Some(selected) = list_state.selected() {
                                let new_selected = if selected == 0 {
                                    tasks.len() - 1
                                } else {
                                    selected - 1
                                };
                                list_state.select(Some(new_selected));
                            }
                        }
                        KeyCode::Char('j') => {
                            if let Some(selected) = list_state.selected() {
                                let new_selected = if selected == tasks.len() - 1 {
                                    0
                                } else {
                                    selected + 1
                                };
                                list_state.select(Some(new_selected));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Terminal closing
    stdout().execute(LeaveAlternateScreen);
    disable_raw_mode()?;
    Ok(())
}
