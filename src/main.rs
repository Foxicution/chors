mod app;
mod errors;
mod ui;

use crate::{
    app::{AppMode, AppState, Task},
    errors::install_hooks,
    ui::ui,
};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::Terminal;

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        set_list_state(app);

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
                KeyCode::Char('a') => {
                    app.mode = AppMode::AddingTask;
                    app.input.clear();
                }
                KeyCode::Char('A') => {
                    app.mode = AppMode::AddingSubtask;
                    app.input.clear();
                }
                KeyCode::Char('c') => {
                    if let Some(task) = app.get_task_mut() {
                        task.completed = !task.completed;
                    }
                }
                KeyCode::Char('k') => navigate_tasks(app, true),
                KeyCode::Char('j') => navigate_tasks(app, false),
                KeyCode::Char('h') => {}
                KeyCode::Char('l') => {}
                _ => {}
            },
            AppMode::AddingTask => match event.code {
                KeyCode::Enter => {
                    app.add_task();
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
            AppMode::AddingSubtask => match event.code {
                KeyCode::Enter => {
                    app.add_subtask();
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

fn set_list_state(app: &mut AppState) {
    if app.nav.is_empty() {
        app.list_state.select(None);
    } else if let Some(selected) = app.selected {
        let index = app.nav.get_index_of(&selected).unwrap_or(0);
        app.list_state.select(Some(index));
    } else {
        app.list_state.select(None);
    }
}

fn navigate_tasks(app: &mut AppState, up: bool) {
    let nav_len = app.nav.len();
    if nav_len == 0 {
        app.selected = None;
        app.list_state.select(None);
        return;
    }

    let new_selected = match app.selected {
        Some(current) => {
            let current_index = app.nav.get_index_of(&current).unwrap_or(0);
            if up {
                if current_index == 0 {
                    nav_len - 1
                } else {
                    current_index - 1
                }
            } else {
                (current_index + 1) % nav_len
            }
        }
        None => 0,
    };

    let (new_selected_id, _) = app.nav.get_index(new_selected).unwrap();
    app.selected = Some(*new_selected_id);
    app.list_state.select(Some(new_selected));
}

fn main() -> Result<()> {
    install_hooks()?;
    let mut terminal = ui::init()?;

    // Initial application state
    let mut app = AppState::new();

    // Run the application
    let result = run_app(&mut terminal, &mut app);

    // Terminal closing
    ui::restore()?;
    result
}
