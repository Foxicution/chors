use std::io::{self, stdout, Stdout};

use crate::app::{AppMode, AppState, Task};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indexmap::IndexMap;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use uuid::Uuid;

type Tui = Terminal<CrosstermBackend<Stdout>>;

// Terminal initialization
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    Ok(terminal)
}

pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn ui(frame: &mut Frame, app: &mut AppState) {
    let size = frame.size();
    let (items, nav) = build_task_list(&app.tasks, Vec::new());
    app.nav = nav;

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Tasks {:?} {:?} {:?} {:?}",
            app.list_state,
            app.nav,
            match app.list_state.selected() {
                Some(s) => Some(&app.nav[s]),
                None => None,
            },
            app.get_task()
        )))
        .highlight_style(Style::default().bg(Color::LightBlue));

    frame.render_stateful_widget(list, size, &mut app.list_state);

    if let AppMode::AddingTask | AppMode::AddingSubtask = app.mode {
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

fn build_task_list(
    tasks: &IndexMap<Uuid, Task>,
    path: Vec<Uuid>,
) -> (Vec<ListItem>, IndexMap<Uuid, Vec<Uuid>>) {
    let mut items = Vec::new();
    let mut nav = IndexMap::new();

    for task in tasks.values() {
        let mut current_path = path.clone();
        current_path.push(task.id);
        nav.insert(task.id, current_path.clone());

        let indent = "  ".repeat(current_path.len() - 1);
        let status = if task.completed { "[x]" } else { "[ ]" };
        items.push(ListItem::new(format!(
            "{}{} {}",
            indent, status, task.description
        )));

        if !task.subtasks.is_empty() {
            let (sub_items, sub_nav) = build_task_list(&task.subtasks, current_path);
            items.extend(sub_items);
            nav.extend(sub_nav);
        }
    }

    (items, nav)
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
