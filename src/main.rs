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

#[derive(Debug, Clone)]
enum AppMode {
    Normal,
    AddingTask,
}

struct AppState {
    tasks: Vec<Task>,
    list_state: ListState,
    mode: AppMode,
    input: String,
}

impl AppState {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            tasks: Vec::new(),
            list_state,
            mode: AppMode::Normal,
            input: String::new(),
        }
    }

    fn update(&mut self, event: event::Event) -> bool {
        match event {
            event::Event::Key(key) if key.kind == KeyEventKind::Press => match self.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('q') => return false,
                    KeyCode::Char('n') => {
                        self.mode = AppMode::AddingTask;
                        self.input.clear();
                    }
                    KeyCode::Char('c') => {
                        if let Some(selected) = self.list_state.selected() {
                            if let Some(task) = self.tasks.get_mut(selected) {
                                task.completed = !task.completed;
                            }
                        }
                    }
                    KeyCode::Char('k') => {
                        if let Some(selected) = self.list_state.selected() {
                            let new_selected = if selected == 0 {
                                self.tasks.len() - 1
                            } else {
                                selected - 1
                            };
                            self.list_state.select(Some(new_selected));
                        }
                    }
                    KeyCode::Char('j') => {
                        if let Some(selected) = self.list_state.selected() {
                            let new_selected = if selected == self.tasks.len() - 1 {
                                0
                            } else {
                                selected + 1
                            };
                            self.list_state.select(Some(new_selected));
                        }
                    }
                    _ => {}
                },
                AppMode::AddingTask => match key.code {
                    KeyCode::Enter => {
                        self.tasks.push(Task::new(&self.input));
                        self.list_state.select(Some(self.tasks.len() - 1));
                        self.mode = AppMode::Normal;
                    }
                    KeyCode::Esc => {
                        self.mode = AppMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        self.input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    _ => {}
                },
            },
            _ => {}
        }
        true
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        let size = frame.size();
        let items: Vec<ListItem> = self
            .tasks
            .iter()
            .map(|task| {
                let status = if task.completed { "[x]" } else { "[ ]" };
                ListItem::new(format!("{} {}", status, task.description))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Tasks"))
            .highlight_style(Style::default().bg(Color::LightBlue));

        frame.render_stateful_widget(list, size, &mut self.list_state.clone());

        if let AppMode::AddingTask = self.mode {
            let area = centered_rect(50, 20, size);
            let input_block = Block::default().borders(Borders::ALL).title("New Task");
            let input_paragraph = Paragraph::new(self.input.as_str())
                .block(input_block)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(input_paragraph, area);
        }
    }
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

    // Initial application state
    let mut state = AppState::new();

    loop {
        // Drawing the UI
        terminal.draw(|frame| {
            state.draw(frame);
        })?;

        // Accepting inputs
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if !state.update(event::Event::Key(key)) {
                    break;
                }
            }
        }
    }

    // Terminal closing
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
