use crate::{
    model::{Mode, Model, Overlay, Task},
    tui::{
        cli::build_cli,
        errors::install_hooks,
        view::{init, restore, ui},
    },
    update::{update, Direction, History, Message},
    utils::VectorUtils,
};
use color_eyre::{eyre::Ok, Result};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use std::{fs, path::Path, time::Duration};

pub fn run() -> Result<()> {
    install_hooks()?;

    let matches = build_cli().get_matches();
    let file_path = matches.get_one::<String>("file");

    let mut terminal = init()?;
    let model = if let Some(file_path) = file_path {
        if Path::new(file_path).exists() {
            let data = fs::read_to_string(file_path)?;
            serde_json::from_str(&data)?
        } else {
            Model::new()
        }
    } else {
        Model::new()
    };

    let model = run_app(&mut terminal, model)?.with_no_message();
    restore()?;

    if let Some(file_path) = file_path {
        let data = serde_json::to_string_pretty(&model)?;
        fs::write(file_path, data)?;
    };

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut model: Model) -> Result<Model> {
    let mut history = History::new(100);

    loop {
        terminal.draw(|f| ui(f, &model))?;

        if !poll_for_event()? {
            continue;
        }

        if let Some(msg) = handle_key_event(&model)? {
            if let Message::Quit = msg {
                return Ok(model);
            }

            model = update(&msg, &model, &mut history);
        }
    }
}

fn keycode_to_message(model: &Model, key: KeyCode, modifiers: KeyModifiers) -> Option<Message> {
    let message = match model.overlay {
        Overlay::None => match model.mode {
            Mode::List => match key {
                KeyCode::Char('q') => Message::Quit,
                KeyCode::Char('j') => Message::Navigate(Direction::Down),
                KeyCode::Char('k') => Message::Navigate(Direction::Up),
                KeyCode::Char('a') => Message::SetOverlay(Overlay::AddingSiblingTask),
                KeyCode::Char('A') => Message::SetOverlay(Overlay::AddingChildTask),
                KeyCode::Char('c') => Message::FlipCompleted(model.get_path()?.to_vec()),
                KeyCode::Char('d') => Message::RemoveTask(model.get_path()?.to_vec()),
                KeyCode::Char('f') => Message::SetOverlay(Overlay::EditFilterCondition),
                KeyCode::Char('u') => Message::Undo,
                KeyCode::Char('U') => Message::Redo,
                _ => return None,
            },
        },
        Overlay::AddingSiblingTask | Overlay::AddingChildTask | Overlay::EditFilterCondition => {
            match key {
                KeyCode::Enter => {
                    let input = model.input.clone();
                    match model.overlay {
                        Overlay::AddingSiblingTask => Message::AddSiblingTask(Task::new(input)),
                        Overlay::AddingChildTask => Message::AddChildTask(Task::new(input)),
                        Overlay::EditFilterCondition => Message::ApplyFilter(input),
                        Overlay::None => unreachable!(),
                    }
                }
                KeyCode::Backspace if modifiers.contains(KeyModifiers::CONTROL) => Message::PopWord,
                KeyCode::Char('w') if modifiers.contains(KeyModifiers::CONTROL) => Message::PopWord,
                KeyCode::Backspace => Message::PopChar,
                KeyCode::Left if modifiers.contains(KeyModifiers::CONTROL) => {
                    Message::JumpWord(Direction::Up)
                }
                KeyCode::Right if modifiers.contains(KeyModifiers::CONTROL) => {
                    Message::JumpWord(Direction::Down)
                }
                KeyCode::Left => Message::Move(Direction::Up),
                KeyCode::Right => Message::Move(Direction::Down),
                KeyCode::Home => Message::JumpStart,
                KeyCode::End => Message::JumpEnd,
                KeyCode::Char(ch) => Message::AddChar(ch),
                KeyCode::Esc => Message::SetOverlay(Overlay::None),
                _ => return None,
            }
        }
    };

    Some(message)
}

fn poll_for_event() -> Result<bool> {
    Ok(poll(Duration::from_millis(16))?)
}

fn handle_key_event(model: &Model) -> Result<Option<Message>> {
    if let Event::Key(key) = read()? {
        if key.kind == KeyEventKind::Press {
            return Ok(keycode_to_message(model, key.code, key.modifiers));
        }
    }
    Ok(None)
}
