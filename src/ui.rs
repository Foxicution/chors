use crate::app::{AppMode, AppState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn ui(frame: &mut Frame, app: &mut AppState) {
    let size = frame.size();
    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .map(|task| {
            let status = if task.completed { "[x]" } else { "[ ]" };
            ListItem::new(format!("{} {}", status, task.description))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(Style::default().bg(Color::LightBlue));

    frame.render_stateful_widget(list, size, &mut app.list_state);

    if let AppMode::AddingTask = app.mode {
        let area = centered_rect(50, 20, size);
        let input_block = Block::default().borders(Borders::ALL).title("New Task");
        let input_paragraph = Paragraph::new(app.input.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(input_paragraph, area);

        // Position the cursor at the end of the input text
        let cursor_x = area.x + app.input.len() as u16 + 1; // +1 for the border offset
        let cursor_y = area.y + 1; // +1 for the border offset
        frame.set_cursor(cursor_x, cursor_y);
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
