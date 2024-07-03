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
                handle_key_event(app, key);
                if let AppMode::Quit = app.mode {
                    return Ok(());
                }
            }
        }
    }
}

fn handle_key_event(app: &mut AppState, event: event::KeyEvent) {
    if event.kind == KeyEventKind::Press {
        match app.mode {
            AppMode::Normal => handle_normal_mode(app, event),
            AppMode::AddingTask => handle_adding_task_mode(app, event),
            AppMode::AddingSubtask => handle_adding_subtask_mode(app, event),
            AppMode::AddingFilterCriterion => handle_adding_filter_mode(app, event),
            AppMode::ViewMode => handle_view_mode(app, event),
            AppMode::DebugOverlay => handle_debug_overlay_mode(app, event),
            AppMode::Navigation => handle_navigation_mode(app, event),
            AppMode::Quit => {}
        }
    }
}

fn handle_normal_mode(app: &mut AppState, event: event::KeyEvent) {
    match event.code {
        KeyCode::Char('q') => app.switch_mode(AppMode::Quit),
        KeyCode::Char('a') => app.switch_mode(AppMode::AddingTask),
        KeyCode::Char('A') => app.switch_mode(AppMode::AddingSubtask),
        KeyCode::Char('v') => app.switch_mode(AppMode::ViewMode),
        KeyCode::Char('f') => app.switch_mode(AppMode::AddingFilterCriterion),
        KeyCode::Char('c') => app.toggle_task_completion(),
        KeyCode::Char('k') => navigate_tasks(app, true),
        KeyCode::Char('j') => navigate_tasks(app, false),
        KeyCode::Char('p') => app.switch_mode(AppMode::DebugOverlay),
        KeyCode::Char('g') => app.switch_mode(AppMode::Navigation),
        _ => {}
    }
}

fn handle_navigation_mode(app: &mut AppState, event: event::KeyEvent) {
    match event.code {
        KeyCode::Char('g') => app.handle_navigation(),
        KeyCode::Char('e') => app.jump_to_end(),
        KeyCode::Char(c) if c.is_ascii_digit() => app.navigation_input.push(c),
        KeyCode::Backspace => {
            app.navigation_input.pop();
        }
        KeyCode::Esc => app.exit_navigation_mode(),
        _ => {}
    }
}

fn handle_adding_task_mode(app: &mut AppState, event: event::KeyEvent) {
    match event.code {
        KeyCode::Enter => {
            app.add_task();
            app.switch_mode(AppMode::Normal);
        }
        KeyCode::Esc => app.switch_mode(AppMode::Normal),
        KeyCode::Char(c) => app.input.push(c),
        KeyCode::Backspace => {
            app.input.pop();
        }
        _ => {}
    }
}

fn handle_adding_subtask_mode(app: &mut AppState, event: event::KeyEvent) {
    match event.code {
        KeyCode::Enter => {
            app.add_subtask();
            app.switch_mode(AppMode::Normal);
        }
        KeyCode::Esc => app.switch_mode(AppMode::Normal),
        KeyCode::Char(c) => app.input.push(c),
        KeyCode::Backspace => {
            app.input.pop();
        }
        _ => {}
    }
}

fn handle_adding_filter_mode(app: &mut AppState, event: event::KeyEvent) {
    match event.code {
        KeyCode::Enter => {
            let input = app.input.clone();
            let parts: Vec<&str> = input.split_whitespace().collect();
            let filters = parts
                .iter()
                .filter_map(|&part| {
                    if part.starts_with("completed") {
                        Some(Filter::Completed(part.ends_with("true")))
                    } else if part.starts_with("tag") {
                        Some(Filter::Tag(part[4..].to_string()))
                    } else if part.starts_with("context") {
                        Some(Filter::Context(part[8..].to_string()))
                    } else {
                        None
                    }
                })
                .collect();
            app.add_filter_list(FilterList { filters });
            app.switch_mode(AppMode::Normal);
        }
        KeyCode::Esc => app.switch_mode(AppMode::Normal),
        KeyCode::Char(c) => app.input.push(c),
        KeyCode::Backspace => {
            app.input.pop();
        }
        _ => {}
    }
}

fn handle_view_mode(app: &mut AppState, event: event::KeyEvent) {
    match event.code {
        KeyCode::Enter => {
            let input = app.input.clone();
            app.save_current_view_as_view(&input);
            app.switch_mode(AppMode::Normal);
        }
        KeyCode::Esc => app.switch_mode(AppMode::Normal),
        KeyCode::Char(c) => app.input.push(c),
        KeyCode::Backspace => {
            app.input.pop();
        }
        _ => {}
    }
}

fn handle_debug_overlay_mode(app: &mut AppState, event: event::KeyEvent) {
    match event.code {
        KeyCode::Char('p') => app.switch_mode(AppMode::Normal),
        KeyCode::Char('j') => app.debug_scroll += 1,
        KeyCode::Char('k') => {
            if app.debug_scroll > 0 {
                app.debug_scroll -= 1
            }
        }
        _ => {}
    }
}

fn set_list_state(app: &mut AppState) {
    if app.nav.is_empty() {
        app.selected = None;
    }
    if let Some(selected) = app.selected {
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

// TODO: add better movement (jump to the start/end, specific line)
// TODO: add task editing (moving up/down a scope, moving in out, yanking and pasting, selecting, etc.)
// TODO: add lists (so that we can have complete separation)
// TODO: add persistance
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
