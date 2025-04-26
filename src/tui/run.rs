use crate::{
    model::{Filter, Form, Mode, Model, Overlay, Task},
    tui::{
        cli::build_cli,
        errors::install_hooks,
        view::{init, restore, ui},
    },
    update::{Direction, History, Message, update},
    utils::VectorUtils,
};
use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, poll, read};
use ratatui::{Terminal, backend::Backend};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub fn run() -> Result<()> {
    install_hooks()?;
    let mut terminal = init()?;

    let matches = build_cli().get_matches();
    let file_path = matches.get_one::<String>("file").map(expand_tilde);

    let model = match file_path.as_ref() {
        Some(path) => {
            if path.exists() {
                let data = fs::read_to_string(path)?;
                serde_json::from_str(&data)?
            } else {
                Model::new()
            }
        }
        None => Model::new(),
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
    let mut redraw = true;

    loop {
        if redraw {
            terminal.draw(|f| ui(f, &model))?;
            redraw = false;
        }

        if !poll_for_event()? {
            continue;
        }

        let event = read()?;
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if let Some(msg) = keycode_to_message(&model, key.code, key.modifiers) {
                    if let Message::Quit = msg {
                        return Ok(model);
                    }

                    model = update(&msg, &model, &mut history);
                    redraw = true;
                }
            }
            Event::Resize(_, _) => {
                redraw = true;
            }
            _ => {}
        }
    }
}

pub struct KeyBind {
    pub keys: &'static [(KeyCode, KeyModifiers)],
    pub modes: &'static [Mode],
    pub action: fn(model: &Model) -> Option<Message>,
    pub description: &'static str,
}

pub const KEYBINDS: &[KeyBind] = &[
    KeyBind {
        keys: &[(KeyCode::Char('q'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::Quit),
        description: "Quit",
    },
    KeyBind {
        keys: &[(KeyCode::Char('j'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::Navigate(Direction::Down)),
        description: "Navigate down",
    },
    KeyBind {
        keys: &[(KeyCode::Char('k'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::Navigate(Direction::Up)),
        description: "Navigate up",
    },
    KeyBind {
        keys: &[(KeyCode::Char('a'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::SetOverlay(Overlay::AddingSiblingTask)),
        description: "Add a task",
    },
    KeyBind {
        keys: &[(KeyCode::Char('A'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::SetOverlay(Overlay::AddingChildTask)),
        description: "Add a subtask",
    },
    KeyBind {
        keys: &[(KeyCode::Char('c'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |model| Some(Message::FlipCompleted(model.get_path()?.to_vec())),
        description: "Toggle task completion",
    },
    KeyBind {
        keys: &[(KeyCode::Char('d'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |model| Some(Message::RemoveTask(model.get_path()?.to_vec())),
        description: "Remove task",
    },
    KeyBind {
        keys: &[(KeyCode::Char('f'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::SetOverlay(Overlay::EditFilterCondition)),
        description: "Edit current filter",
    },
    KeyBind {
        keys: &[(KeyCode::Char('F'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::SetOverlay(Overlay::AddingFilter)),
        description: "Save current filter",
    },
    KeyBind {
        keys: &[(KeyCode::Char(' '), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::SetOverlay(Overlay::SelectingFilter)),
        description: "Select a filter",
    },
    KeyBind {
        keys: &[(KeyCode::Char('u'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::Undo),
        description: "Undo last action",
    },
    KeyBind {
        keys: &[(KeyCode::Char('U'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::Redo),
        description: "Redo last action",
    },
    KeyBind {
        keys: &[(KeyCode::Char('?'), KeyModifiers::NONE)],
        modes: &[Mode::List],
        action: |_| Some(Message::SetOverlay(Overlay::Help)),
        description: "Show help",
    },
];

fn keycode_to_message(model: &Model, key: KeyCode, modifiers: KeyModifiers) -> Option<Message> {
    let message = match model.overlay {
        Overlay::None => {
            // Drop the SHIFT bit (add later if needed for something)
            let mods = modifiers & !KeyModifiers::SHIFT;
            for kb in KEYBINDS {
                if kb.modes.contains(&model.mode) && kb.keys.contains(&(key, mods)) {
                    return (kb.action)(model);
                }
            }
            return None;
        }
        Overlay::Help => match key {
            KeyCode::Esc | KeyCode::Char('?') => {
                return Some(Message::SetOverlay(Overlay::None));
            }
            _ => return None,
        },
        Overlay::SelectingFilter => match key {
            KeyCode::Char(ch) if ch.is_ascii_digit() => return None,
            _ => return None,
        },
        Overlay::AddingSiblingTask
        | Overlay::AddingChildTask
        | Overlay::EditFilterCondition
        | Overlay::AddingFilter => match key {
            KeyCode::Enter => {
                let input = model.input.text.clone();
                match model.overlay {
                    Overlay::AddingSiblingTask => Message::AddSiblingTask(Task::new(input)),
                    Overlay::AddingChildTask => Message::AddChildTask(Task::new(input)),
                    Overlay::EditFilterCondition => Message::ApplyFilter(input),
                    Overlay::AddingFilter => {
                        Message::AddFilter(Filter::new(input, model.current_filter.clone()))
                    }
                    _ => unreachable!(),
                }
            }
            KeyCode::Backspace if modifiers.contains(KeyModifiers::CONTROL) => {
                Message::SetInput(model.input.with_popped_word())
            }
            KeyCode::Char('w') if modifiers.contains(KeyModifiers::CONTROL) => {
                Message::SetInput(model.input.with_popped_word())
            }
            KeyCode::Backspace => Message::SetInput(model.input.with_popped_char()),
            KeyCode::Left if modifiers.contains(KeyModifiers::CONTROL) => {
                Message::SetInput(model.input.with_cursor_jump_word(&Direction::Up))
            }
            KeyCode::Right if modifiers.contains(KeyModifiers::CONTROL) => {
                Message::SetInput(model.input.with_cursor_jump_word(&Direction::Down))
            }
            KeyCode::Left => Message::SetInput(model.input.with_cursor_move(&Direction::Up)),
            KeyCode::Right => Message::SetInput(model.input.with_cursor_move(&Direction::Down)),
            KeyCode::Home => Message::SetInput(model.input.with_cursor(0)),
            KeyCode::End => Message::SetInput(model.input.with_cursor(model.input.text.len())),
            KeyCode::Char(ch) => Message::SetInput(model.input.with_inserted_char(ch)),
            KeyCode::Esc => Message::SetOverlay(Overlay::None),
            _ => return None,
        },
    };

    Some(message)
}

fn poll_for_event() -> Result<bool> {
    Ok(poll(EVENT_POLL_INTERVAL)?)
}

/// Expands `~` to the user's home directory using `std::env`.
fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    if let Ok(stripped) = path.strip_prefix("~") {
        if let Ok(home) = env::var("HOME") {
            return Path::new(&home).join(stripped);
        } else if let Ok(user_profile) = env::var("USERPROFILE") {
            // On Windows, use `USERPROFILE` as the fallback for the home directory
            return Path::new(&user_profile).join(stripped);
        }
    };
    path.to_path_buf()
}
