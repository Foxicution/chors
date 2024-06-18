use crate::app::{AppMode, AppState, Task, View};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indexmap::IndexMap;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{
    collections::HashSet,
    io::{self, stdout, Stdout},
};
use uuid::Uuid;

type Tui = Terminal<CrosstermBackend<Stdout>>;

struct UIList<'a> {
    pub items: Vec<ListItem<'a>>,
    pub nav: IndexMap<Uuid, Vec<Uuid>>,
    pub tags: HashSet<String>,
    pub contexts: HashSet<String>,
}

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
    let ui_list = build_task_list(&app.tasks, Vec::new(), &app.current_view, false, 0);
    app.nav = ui_list.nav;
    app.tags = ui_list.tags;
    app.contexts = ui_list.contexts;

    let list = List::new(ui_list.items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(Style::default().bg(Color::Indexed(8)));

    frame.render_stateful_widget(list, size, &mut app.list_state);

    if let AppMode::DebugOverlay = app.mode {
        let debug_area = centered_rect(50, 50, size);
        let debug_block = Block::default()
            .borders(Borders::ALL)
            .title("Debug Overlay");
        let debug_content = format!("{:#?}", app); // Display app state
        let debug_paragraph = Paragraph::new(debug_content)
            .block(debug_block)
            .style(Style::default().fg(Color::Red))
            .scroll((app.debug_scroll, 0));
        frame.render_widget(debug_paragraph, debug_area);
    }

    if let AppMode::ViewMode = app.mode {
        let area = centered_rect(50, 20, size);
        let input_block = Block::default().borders(Borders::ALL).title("View Name");
        let input_paragraph = Paragraph::new(app.input.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(input_paragraph, area);

        let cursor_x = area.x + app.input.len() as u16 + 1;
        let cursor_y = area.y + 1;
        frame.set_cursor(cursor_x, cursor_y);
    }

    if let AppMode::AddingTask | AppMode::AddingSubtask | AppMode::AddingFilterCriterion = app.mode
    {
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

fn build_task_list<'a>(
    tasks: &'a IndexMap<Uuid, Task>,
    path: Vec<Uuid>,
    view: &'a View,
    parent_match: bool,
    depth: usize,
) -> UIList<'a> {
    let mut items = Vec::new();
    let mut nav = IndexMap::new();
    let mut tags = HashSet::new();
    let mut contexts = HashSet::new();

    for task in tasks.values() {
        let mut current_path = path.clone();
        current_path.push(task.id);

        if view.matches(task) | parent_match {
            nav.insert(task.id, current_path.clone());

            add_task_to_ui_list(task, &mut items, &mut tags, &mut contexts, depth);
            let sub = build_task_list(&task.subtasks, current_path, view, true, depth + 1);
            items.extend(sub.items);
            nav.extend(sub.nav);
            tags.extend(sub.tags);
            contexts.extend(sub.contexts);
        } else {
            let sub = build_task_list(&task.subtasks, current_path, view, false, depth);
            if !sub.items.is_empty() {
                // let mut current_path = path.clone();
                // current_path.push(task.id);
                // nav.insert(task.id, current_path.clone());
                // add_task_to_ui_list(task, &mut items, &mut tags, &mut contexts, 0);
                items.extend(sub.items);
                nav.extend(sub.nav);
                tags.extend(sub.tags);
                contexts.extend(sub.contexts);
            }
        }
    }

    UIList {
        items,
        nav,
        tags,
        contexts,
    }
}

fn add_task_to_ui_list<'a>(
    task: &'a Task,
    items: &mut Vec<ListItem<'a>>,
    tags: &mut HashSet<String>,
    contexts: &mut HashSet<String>,
    indent_level: usize,
) {
    let indent = "  ".repeat(indent_level);
    let status = if task.completed {
        Span::styled("[x]", Style::default().fg(Color::Green))
    } else {
        Span::styled("[ ]", Style::default().fg(Color::Yellow))
    };
    let mut description_spans = Vec::new();
    description_spans.push(Span::raw(format!("{} ", indent)));
    description_spans.push(status);
    description_spans.push(Span::raw(" "));

    for word in task.description.split_whitespace() {
        if word.starts_with('#') {
            tags.insert(word.to_string());
            description_spans.push(Span::styled(word, Style::default().fg(Color::Magenta)));
        } else if word.starts_with('@') {
            contexts.insert(word.to_string());
            description_spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
        } else {
            description_spans.push(Span::raw(word));
        }
        description_spans.push(Span::raw(" "));
    }

    items.push(ListItem::new(Line::from(description_spans)));
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
