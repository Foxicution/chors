use crate::{
    model::{DisplayMessage, Mode, Model, Overlay},
    tui::style::{style_input_task, style_task},
    utils::VectorUtils,
};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

use super::style::style_input_filter;

type Tui = Terminal<CrosstermBackend<io::Stdout>>;

const INFO_HEIGHT: u16 = 1;
const INPUT_HEIGHT: u16 = 1;
const STATUS_HEIGHT: u16 = INFO_HEIGHT + INPUT_HEIGHT;

pub fn ui(frame: &mut Frame, model: &Model) {
    let size = frame.size();
    let available_height = size.height.saturating_sub(STATUS_HEIGHT);
    let available_size = Rect {
        height: available_height,
        ..size
    };

    match model.mode {
        Mode::List => render_list_mode(frame, model, available_size),
        Mode::List => render_mode_list(frame, model, available_size),
    }

    match model.overlay {
        Overlay::SelectingFilter => render_overlay_selectingfilter(frame, model, available_size),
        _ => {}
    }
    render_taskbar(frame, model, size);
}

fn render_overlay_selectingfilter(frame: &mut Frame, model: &Model, size: Rect) {}

fn render_mode_list(frame: &mut Frame, model: &Model, size: Rect) {
    if model.filtered_tasks.is_empty() {
        // Step 1: Calculate the dimensions for a centered message
        let message = "Wow so Empty!\nAdd some todos to get started!";
        let message_height = 2; // Adjust this to match the number of lines in the message
        let y_offset = (size.height / 2).saturating_sub(message_height); // Center vertically
        let x_offset = size.width / 4; // Adjust for a bit of padding on the sides

        // Step 2: Define a centered rectangle for the message
        let centered_rect = Rect::new(
            x_offset,
            y_offset,
            size.width / 2,     // Make the message take half the screen width
            message_height * 2, // Approximate height for the message box
        );

        // Step 3: Create and style the paragraph for the message
        let empty_message = Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));

        // Render the paragraph at the calculated position
        frame.render_widget(empty_message, centered_rect);
    } else {
        let selected_task_index = match model.selected_task {
            Some(id) => model.filtered_tasks.get_index(&id),
            None => None,
        };

        let task_list = model
            .filtered_tasks
            .iter()
            .enumerate()
            .map(|(index, (_, path))| {
                let task = model
                    .get_task(&path.to_vec())
                    .expect("Should find a task from filtered tasks!");
                let ident = path.len();
                let line_number_style = if Some(index) == selected_task_index {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let line_number = Span::styled(format!("{:>3} ", index + 1), line_number_style);
                let mut line_spans = vec![line_number];

                line_spans.extend(style_task(task, ident));
                ListItem::new(Line::from(line_spans))
            });

        let list = List::new(task_list).highlight_style(Style::default().bg(Color::Indexed(8)));
        let mut selection_state = ListState::default().with_selected(selected_task_index);

        frame.render_stateful_widget(list, size, &mut selection_state);
    }
}

fn render_taskbar(frame: &mut Frame, model: &Model, size: Rect) {
    let info_area = Rect::new(size.x, size.height - STATUS_HEIGHT, size.width, INFO_HEIGHT);
    let input_area = Rect::new(size.x, size.height - INPUT_HEIGHT, size.width, INPUT_HEIGHT);

    let info_paragraph = Paragraph::new(format!(
        " <{}>",
        model
            .get_selected_filter()
            .map(|f| f.name.as_str())
            .unwrap_or("")
    ))
    .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let input_paragraph = match model.overlay {
        Overlay::None | Overlay::SelectingFilter => match model.message.clone() {
            DisplayMessage::None => Paragraph::new(""),
            DisplayMessage::Success(msg) => {
                Paragraph::new(msg).style(Style::default().fg(Color::Green))
            }
            DisplayMessage::Error(msg) => {
                Paragraph::new(msg).style(Style::default().fg(Color::Red))
            }
        },
        Overlay::AddingSiblingTask | Overlay::AddingChildTask => {
            frame.set_cursor(model.input.cursor as u16, input_area.y);
            Paragraph::new(Line::from(style_input_task(&model.input.text)))
        }
        Overlay::EditFilterCondition => {
            frame.set_cursor(model.input.cursor as u16, input_area.y);
            Paragraph::new(Line::from(style_input_filter(&model.input.text)))
        }
        Overlay::AddingFilter => {
            frame.set_cursor(model.input.cursor as u16, input_area.y);
            Paragraph::new(Line::from(model.input.text.clone()))
        }
    };

    frame.render_widget(info_paragraph, info_area);
    frame.render_widget(input_paragraph, input_area);
}

pub fn init() -> io::Result<Tui> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;
    Ok(terminal)
}

pub fn restore() -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
