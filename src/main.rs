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
