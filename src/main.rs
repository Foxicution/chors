mod app;
mod errors;
mod ui;

use crate::{
    app::{AppMode, AppState, Filter, FilterList},
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
                KeyCode::Char('v') => {
                    app.mode = AppMode::ViewMode;
                    app.input.clear();
                }
                KeyCode::Char('f') => {
                    app.mode = AppMode::AddingFilterCriterion;
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
                KeyCode::Char('p') => {
                    app.mode = AppMode::DebugOverlay;
                }
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
            AppMode::AddingFilterCriterion => match event.code {
                KeyCode::Enter => {
                    let input = app.input.clone(); // Clone input to avoid borrow issues
                    let parts: Vec<&str> = input.split_whitespace().collect();
                    let mut filters = Vec::new();
                    for part in parts {
                        if part.starts_with("completed") {
                            let value = part.split('=').nth(1).unwrap_or("false") == "true";
                            filters.push(Filter::Completed(value));
                        } else if part.starts_with("tag") {
                            let tag = part.split('=').nth(1).unwrap_or("").to_string();
                            filters.push(Filter::Tag(tag));
                        } else if part.starts_with("context") {
                            let context = part.split('=').nth(1).unwrap_or("").to_string();
                            filters.push(Filter::Context(context));
                        }
                    }
                    app.add_filter_list(FilterList { filters });
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
            AppMode::ViewMode => match event.code {
                KeyCode::Enter => {
                    let input = app.input.clone(); // Clone input to avoid borrow issues
                    app.save_current_view_as_view(&input);
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
            AppMode::DebugOverlay => match event.code {
                KeyCode::Char('p') => {
                    app.mode = AppMode::Normal;
                }
                KeyCode::Char('j') => {
                    app.debug_scroll += 1;
                }
                KeyCode::Char('k') => {
                    if app.debug_scroll > 0 {
                        app.debug_scroll -= 1;
                    }
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
        let index = app.nav.get_index_of(&selected).unwrap_or(app.nav.len() - 1);
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

// TODO: add filters and filtered views
// TODO: add lists (so that we can have complete separation)
// TODO: add persistance
// TODO: add movement operations
// TODO: improve ui visibility (colors, etc. inspiration dooit)
// TODO: add the ability to host from a server
// TODO: add a web ui with iced so I can use this on the phone...
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
