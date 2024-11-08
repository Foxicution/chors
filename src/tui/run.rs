use crate::{
    model::{Mode, Model, Task},
    tui::{
        errors::install_hooks,
        view::{init, restore, ui},
    },
    update::{update, Direction, History, Message},
    utils::VectorUtils,
};
use color_eyre::{eyre::Ok, Result};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind};
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

fn keycode_to_message(model: &Model, key: KeyCode) -> Option<Message> {
    let message = match model.mode {
        Mode::List => match key {
            KeyCode::Char('q') => Message::SwitchMode(Mode::Quit),
            KeyCode::Char('j') => Message::Navigate(Direction::Down),
            KeyCode::Char('k') => Message::Navigate(Direction::Up),
            KeyCode::Char('c') => Message::FlipCompleted(model.get_path()?.to_vec()),
            _ => return None,
        },
        Mode::Quit => return None,
    };

    Some(message)
}

fn poll_for_event() -> Result<bool> {
    Ok(poll(Duration::from_millis(16))?)
}

fn handle_key_event(model: &Model) -> Result<Option<Message>> {
    if let Event::Key(key) = read()? {
        if key.kind == KeyEventKind::Press {
            return Ok(keycode_to_message(model, key.code));
        }
    }
    Ok(None)
}
