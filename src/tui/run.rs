use crate::{
    model::{Mode, Model, Overlay, Task},
    tui::{
        errors::install_hooks,
        view::{init, restore, ui},
    },
    update::{update, Direction, History, Message},
    utils::VectorUtils,
};
use color_eyre::{eyre::Ok, Result};
use crossterm::event::{poll, read, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use std::time::Duration;

pub fn run() -> Result<()> {
    install_hooks()?;
    let mut terminal = init()?;

    let model = Model::new()
        .with_sibling_task(Task::new("this is a taest task"))
        .unwrap()
        .with_sibling_task(Task::new("This is another test task"))
        .unwrap();
    let _result = run_app(&mut terminal, model);

    restore()?;
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
            model = update(&msg, &model, &mut history);

            if let Mode::Quit = model.mode {
                return Ok(model);
            }
        }
    }
}

fn keycode_to_message(model: &Model, key: KeyCode, modifiers: KeyModifiers) -> Option<Message> {
    let message = match model.overlay {
        Overlay::None => match model.mode {
            Mode::List => match key {
                KeyCode::Char('q') => Message::SetMode(Mode::Quit),
                KeyCode::Char('j') => Message::Navigate(Direction::Down),
                KeyCode::Char('k') => Message::Navigate(Direction::Up),
                KeyCode::Char('a') => Message::SetOverlay(Overlay::AddingSiblingTask),
                KeyCode::Char('A') => Message::SetOverlay(Overlay::AddingChildTask),
                KeyCode::Char('c') => Message::FlipCompleted(model.get_path()?.to_vec()),
                KeyCode::Char('u') => Message::Undo,
                KeyCode::Char('U') => Message::Redo,
                _ => return None,
            },
            Mode::Quit => return None,
        },
        Overlay::AddingSiblingTask | Overlay::AddingChildTask => match key {
            KeyCode::Enter => {
                let task = Task::new(model.input.clone());
                match model.overlay {
                    Overlay::AddingSiblingTask => Message::AddSiblingTask(task),
                    Overlay::AddingChildTask => Message::AddChildTask(task),
                    Overlay::None => return None,
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
        },
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
