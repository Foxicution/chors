use crate::{
    model::{Mode, Model},
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
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

type Tui = Terminal<CrosstermBackend<io::Stdout>>;

const INFO_HEIGHT: u16 = 1;
const INPUT_HEIGHT: u16 = 1;
const STATUS_HEIGHT: u16 = INFO_HEIGHT + INPUT_HEIGHT;

pub fn ui(frame: &mut Frame, model: &Model) {
    let size = frame.size();
    let available_height = size.height.saturating_sub(STATUS_HEIGHT);

    match model.mode {
        Mode::List => render_list_mode(frame, model, size),
        Mode::Quit => {}
    }

    render_taskbar(frame, model, size);
}

fn render_list_mode(frame: &mut Frame, model: &Model, size: Rect) {
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
        let task_list = model.filtered_tasks.iter().map(|(_, path)| {
            let task = model
                .get_task(&path.to_vec())
                .expect("Should find a task from filtered tasks!");
            let ident = "  ".repeat(path.len());
            let status = if task.completed.is_some() {
                Span::styled("[x]", Style::default().fg(Color::Green))
            } else {
                Span::styled("[ ]", Style::default().fg(Color::Yellow))
            };
            let mut description_spans = Vec::new();
            description_spans.push(Span::raw(ident));
            description_spans.push(status);
            description_spans.push(Span::raw(" "));

            for word in task.description.split_whitespace() {
                if word.starts_with('#') {
                    description_spans.push(Span::styled(word, Style::default().fg(Color::Magenta)));
                } else if word.starts_with('@') {
                    description_spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
                } else {
                    description_spans.push(Span::raw(word));
                }
                description_spans.push(Span::raw(" "));

                let total_subtasks = task.subtasks.len();
                if total_subtasks > 0 {
                    let completed_subtasks = task
                        .subtasks
                        .iter()
                        .filter(|(_, t)| t.completed.is_some())
                        .count();
                    let color = if completed_subtasks == total_subtasks {
                        Color::Green
                    } else {
                        Color::Yellow
                    };
                    description_spans.push(Span::styled(
                        format!("[{}/{}]", completed_subtasks, total_subtasks),
                        Style::default().fg(color),
                    ));
                }
            }
            ListItem::new(Line::from(description_spans))
        });

        let list = List::new(task_list).highlight_style(Style::default().bg(Color::Indexed(8)));
        let selected_task_index = match model.selected_task {
            Some(id) => model.filtered_tasks.get_index(&id),
            None => None,
        };
        let mut selection_state = ListState::default().with_selected(selected_task_index);

        frame.render_stateful_widget(list, size, &mut selection_state);
    }
}

fn render_taskbar(frame: &mut Frame, model: &Model, size: Rect) {
    let info_area = Rect::new(size.x, size.height - STATUS_HEIGHT, size.width, INFO_HEIGHT);
    let input_area = Rect::new(size.x, size.height - INPUT_HEIGHT, size.width, INPUT_HEIGHT);

    let info_paragraph = Paragraph::new(Span::from(" Test!"))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(info_paragraph, info_area);
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
