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
use model::Overlay;
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
    match model.overlay {
        Overlay::None => match model.mode {
            Mode::List => match key {
                KeyCode::Char('q') => Msg::SwitchMode(Mode::Quit),
                KeyCode::Char('a') => Msg::SetOverlay(Overlay::AddingTask),
                KeyCode::Char('A') => Msg::SetOverlay(Overlay::AddingSubtask),
                KeyCode::Char('v') => Msg::SetOverlay(Overlay::View),
                KeyCode::Char('f') => Msg::SetOverlay(Overlay::AddingFilterCriterion),
                KeyCode::Char('c') => Msg::ToggleTaskCompletion,
                KeyCode::Char('k') => Msg::NavigateTasks(Direction::Up),
                KeyCode::Char('j') => Msg::NavigateTasks(Direction::Down),
                KeyCode::Char('p') => Msg::SetOverlay(Overlay::Debug),
                KeyCode::Char('g') => Msg::SetOverlay(Overlay::Navigation),
                KeyCode::Char('C') => Msg::SwitchMode(Mode::Calendar),
                KeyCode::Char('?') => Msg::SetOverlay(Overlay::Help),
                _ => Msg::NoOp,
            },
            Mode::Calendar => match key {
                KeyCode::Char('C') => Msg::SwitchMode(Mode::List),
                _ => Msg::NoOp,
            },
            Mode::Quit => Msg::Quit,
        },
        Overlay::AddingTask | Overlay::AddingSubtask | Overlay::AddingFilterCriterion => {
            match key {
                KeyCode::Enter => {
                    if let Overlay::AddingTask = model.overlay {
                        Msg::AddTask
                    } else if let Overlay::AddingSubtask = model.overlay {
                        Msg::AddSubtask
                    } else {
                        Msg::AddFilterCriterion
                    }
                }
                KeyCode::Esc => Msg::SetOverlay(Overlay::None),
                KeyCode::Char(c) => Msg::PushChar(c),
                KeyCode::Backspace => Msg::PopChar,
                _ => Msg::NoOp,
            }
        }
        Overlay::View => match key {
            KeyCode::Enter => Msg::SaveCurrentView(model.input.clone()),
            KeyCode::Esc => Msg::SetOverlay(Overlay::None),
            KeyCode::Char(c) => Msg::PushChar(c),
            KeyCode::Backspace => Msg::PopChar,
            _ => Msg::NoOp,
        },
        Overlay::Debug => match key {
            KeyCode::Char('p') => Msg::SetOverlay(Overlay::None),
            KeyCode::Char('j') => Msg::ScrollDebug(Direction::Down),
            KeyCode::Char('k') => Msg::ScrollDebug(Direction::Up),
            _ => Msg::NoOp,
        },
        Overlay::Navigation => match key {
            KeyCode::Char('g') => Msg::HandleNavigation,
            KeyCode::Char('e') => Msg::JumpToEnd,
            KeyCode::Char(c) if c.is_ascii_digit() => Msg::PushChar(c),
            KeyCode::Backspace => Msg::PopChar,
            KeyCode::Esc => Msg::SetOverlay(Overlay::None),
            _ => Msg::NoOp,
        },
        Overlay::Help => match key {
            KeyCode::Esc => Msg::SetOverlay(Overlay::None),
            _ => Msg::NoOp,
        },
    }
}

// TODO: add a calendar and time to tasks
// TODO: add task editing (moving up/down a scope, moving in out, yanking and pasting, selecting, etc.)
// TODO: add lists (so that we can have complete separation)
// TODO: improve ui visibility (colors, etc. inspiration dooit)
// TODO: add a web ui with iced so I can use this on the phone...
// TODO: add the ability to host from a server
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
            model.mode = Mode::List;
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
