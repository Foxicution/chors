mod errors;
mod model;
mod update;
mod view;

use crate::{
    errors::install_hooks,
    model::{Direction, Mode, Model, Msg},
    update::update,
};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::Terminal;

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    model: &mut Model,
) -> Result<()> {
    loop {
        terminal.draw(|f| view::ui(f, model))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let msg = key_event_to_msg(model, key.code);
                    update(msg, model);
                    if let Mode::Quit = model.mode {
                        return Ok(());
                    }
                }
            }
        }
    }
}

fn key_event_to_msg(model: &Model, key: KeyCode) -> Msg {
    match model.mode {
        Mode::Normal => match key {
            KeyCode::Char('q') => Msg::SwitchMode(Mode::Quit),
            KeyCode::Char('a') => Msg::SwitchMode(Mode::AddingTask),
            KeyCode::Char('A') => Msg::SwitchMode(Mode::AddingSubtask),
            KeyCode::Char('v') => Msg::SwitchMode(Mode::View),
            KeyCode::Char('f') => Msg::SwitchMode(Mode::AddingFilterCriterion),
            KeyCode::Char('c') => Msg::ToggleTaskCompletion,
            KeyCode::Char('k') => Msg::NavigateTasks(Direction::Up),
            KeyCode::Char('j') => Msg::NavigateTasks(Direction::Down),
            KeyCode::Char('p') => Msg::SwitchMode(Mode::DebugOverlay),
            KeyCode::Char('g') => Msg::SwitchMode(Mode::Navigation),
            _ => Msg::NoOp,
        },
        Mode::AddingTask | Mode::AddingSubtask | Mode::AddingFilterCriterion => match key {
            KeyCode::Enter => {
                if let Mode::AddingTask = model.mode {
                    Msg::AddTask
                } else if let Mode::AddingSubtask = model.mode {
                    Msg::AddSubtask
                } else {
                    Msg::AddFilterCriterion
                }
            }
            KeyCode::Esc => Msg::SwitchMode(Mode::Normal),
            KeyCode::Char(c) => Msg::PushChar(c),
            KeyCode::Backspace => Msg::PopChar,
            _ => Msg::NoOp,
        },
        Mode::View => match key {
            KeyCode::Enter => Msg::SaveCurrentView(model.input.clone()),
            KeyCode::Esc => Msg::SwitchMode(Mode::Normal),
            KeyCode::Char(c) => Msg::PushChar(c),
            KeyCode::Backspace => Msg::PopChar,
            _ => Msg::NoOp,
        },
        Mode::DebugOverlay => match key {
            KeyCode::Char('p') => Msg::SwitchMode(Mode::Normal),
            KeyCode::Char('j') => Msg::ScrollDebug(Direction::Down),
            KeyCode::Char('k') => Msg::ScrollDebug(Direction::Up),
            _ => Msg::NoOp,
        },
        Mode::Navigation => match key {
            KeyCode::Char('g') => Msg::HandleNavigation,
            KeyCode::Char('G') => Msg::JumpToEnd,
            KeyCode::Char(c) if c.is_ascii_digit() => Msg::PushChar(c),
            KeyCode::Backspace => Msg::PopChar,
            KeyCode::Esc => Msg::SwitchMode(Mode::Normal),
            _ => Msg::NoOp,
        },
        Mode::Quit => Msg::Quit,
    }
}

fn main() -> Result<()> {
    install_hooks()?;
    let mut terminal = view::init()?;

    // Initial application state
    let mut model = Model::new();

    // Run the application
    let result = run_app(&mut terminal, &mut model);

    // Terminal closing
    view::restore()?;
    result
}
