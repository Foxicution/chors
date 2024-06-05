mod app;
mod ui;

use crate::app::{AppMode, AppState, Task};
use crate::ui::ui;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if !handle_key_event(app, key) {
                    return Ok(());
                }
            }
        }
    }
}

fn handle_key_event(app: &mut AppState, event: event::KeyEvent) -> bool {
    if event.kind == KeyEventKind::Press {
        match app.mode {
            AppMode::Normal => match event.code {
                KeyCode::Char('q') => return false,
                KeyCode::Char('n') => {
                    app.mode = AppMode::AddingTask;
                    app.input.clear();
                }
                KeyCode::Char('c') => {
                    if let Some(selected) = app.list_state.selected() {
                        if let Some(task) = app.tasks.get_mut(selected) {
                            task.completed = !task.completed;
                        }
                    }
                }
                KeyCode::Char('k') => {
                    if let Some(selected) = app.list_state.selected() {
                        let new_selected = if selected == 0 {
                            app.tasks.len() - 1
                        } else {
                            selected - 1
                        };
                        app.list_state.select(Some(new_selected));
                    }
                }
                KeyCode::Char('j') => {
                    if let Some(selected) = app.list_state.selected() {
                        let new_selected = if selected == app.tasks.len() - 1 {
                            0
                        } else {
                            selected + 1
                        };
                        app.list_state.select(Some(new_selected));
                    }
                }
                _ => {}
            },
            AppMode::AddingTask => match event.code {
                KeyCode::Enter => {
                    app.tasks.push(Task::new(&app.input));
                    app.list_state.select(Some(app.tasks.len() - 1));
                    app.mode = AppMode::Normal;
                }
                KeyCode::Esc => {
                    app.mode = AppMode::Normal;
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                _ => {}
            },
        }
    };
    true
}

fn main() -> io::Result<()> {
    // Terminal initialization
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Initial application state
    let mut app = AppState::new();

    // Run the application
    let result = run_app(&mut terminal, &mut app);

    // Terminal closing
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    result
}
