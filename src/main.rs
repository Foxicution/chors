mod cli;
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
use std::{fs, path::Path};

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
            KeyCode::Char('e') => Msg::JumpToEnd,
            KeyCode::Char(c) if c.is_ascii_digit() => Msg::PushChar(c),
            KeyCode::Backspace => Msg::PopChar,
            KeyCode::Esc => Msg::SwitchMode(Mode::Normal),
            _ => Msg::NoOp,
        },
        Mode::Quit => Msg::Quit,
    }
}

// TODO: add persistance
// TODO: add task editing (moving up/down a scope, moving in out, yanking and pasting, selecting, etc.)
// TODO: add lists (so that we can have complete separation)
// TODO: improve ui visibility (colors, etc. inspiration dooit)
// TODO: add the ability to host from a server
// TODO: add a web ui with iced so I can use this on the phone...
fn main() -> Result<()> {
    install_hooks()?;

    let matches = cli::build_cli().get_matches();
    let file_path = matches.get_one::<String>("file");

    let mut terminal = view::init()?;

    // Load application state
    let mut model = if let Some(file_path) = file_path {
        if Path::new(file_path).exists() {
            let data = fs::read_to_string(file_path)?;
            let mut model: Model = serde_json::from_str(&data)?;
            model.mode = Mode::Normal;
            model
        } else {
            Model::new()
        }
    } else {
        Model::new()
    };

    // Run the application
    let result = run_app(&mut terminal, &mut model);

    // Terminal closing
    view::restore()?;

    // Save application state if a file path was provided
    if let Some(file_path) = file_path {
        let data = serde_json::to_string_pretty(&model)?;
        fs::write(file_path, data)?;
    }

    result
}
