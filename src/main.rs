#![allow(unused)]

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use indexmap::IndexMap;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io::{stdout, Result, Write};
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

enum AppMode {
    Normal,
    AddingTask,
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn main() -> Result<()> {
    // Terminal initialization
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Main loop
    let mut tasks: Vec<Task> = Vec::new();
    let mut list_state = ListState::default();
    list_state.select(Some(0));
    let mut mode = AppMode::Normal;
    let mut input = String::new();

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
                .highlight_style(Style::default().bg(Color::LightBlue));

            frame.render_stateful_widget(list, size, &mut list_state);

            if let AppMode::AddingTask = mode {
                let area = centered_rect(50, 20, size);
                let input_block = Block::default().borders(Borders::ALL).title("New Task");
                let input_paragraph = Paragraph::new(input.as_str())
                    .block(input_block)
                    .style(Style::default().fg(Color::Yellow));
                frame.render_widget(input_paragraph, area);
            }
        })?;

        // Accepting inputs
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match mode {
                        AppMode::Normal => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('n') => {
                                mode = AppMode::AddingTask;
                                input.clear();
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
                        },
                        AppMode::AddingTask => match key.code {
                            KeyCode::Enter => {
                                tasks.push(Task::new(&input));
                                list_state.select(Some(tasks.len() - 1));
                                mode = AppMode::Normal;
                            }
                            KeyCode::Esc => {
                                mode = AppMode::Normal;
                            }
                            KeyCode::Char(c) => {
                                input.push(c);
                            }
                            KeyCode::Backspace => {
                                input.pop();
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    // Terminal closing
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
