use crate::model::{Mode, Model, Overlay, Task, View};
use chrono::Datelike;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indexmap::IndexMap;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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

pub fn ui(frame: &mut Frame, app: &mut Model) {
    let size = frame.size();

    match app.mode {
        Mode::List => render_list_mode(frame, app, size),
        Mode::Calendar => render_calendar_mode(frame, app, size),
        Mode::Quit => {}
    }

    match app.overlay {
        Overlay::None => {}
        Overlay::AddingTask | Overlay::AddingSubtask | Overlay::AddingFilterCriterion => {
            render_input_overlay(frame, app, size)
        }
        Overlay::View => render_view_overlay(frame, app, size),
        Overlay::Navigation => render_navigation_overlay(frame, app, size),
        Overlay::Help => render_help_overlay(frame, size),
        Overlay::Debug => render_debug_overlay(frame, size),
    }
}

fn render_list_mode(frame: &mut Frame, app: &mut Model, size: Rect) {
    let ui_list = build_task_list(&app.tasks, Vec::new(), &app.current_view, false, 0);
    app.nav = ui_list.nav;
    app.tags = ui_list.tags;
    app.contexts = ui_list.contexts;

    let list = List::new(ui_list.items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(Style::default().bg(Color::Indexed(8)));

    frame.render_stateful_widget(list, size, &mut app.list_state);
}

fn render_input_overlay(frame: &mut Frame, app: &Model, size: Rect) {
    let area = centered_rect(50, 20, size);
    let input_block = Block::default().borders(Borders::ALL).title("New Task");
    let input_paragraph = Paragraph::new(app.input.as_str())
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input_paragraph, area);

    let cursor_x = area.x + app.input.len() as u16 + 1;
    let cursor_y = area.y + 1;
    frame.set_cursor(cursor_x, cursor_y);
}

fn render_view_overlay(frame: &mut Frame, app: &Model, size: Rect) {
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

fn render_navigation_overlay(frame: &mut Frame, app: &Model, size: Rect) {
    let navigation_width = 30;
    let navigation_height = 6;
    let area = Rect::new(
        size.width.saturating_sub(navigation_width + 1),
        size.height.saturating_sub(navigation_height + 1),
        navigation_width,
        navigation_height,
    );

    let navigation_block = Block::default().borders(Borders::ALL).title("Navigation");
    let navigation_text = vec![
        Line::from(vec![
            Span::raw("Go to line: "),
            Span::styled(&app.navigation_input, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(Span::raw("Options:")),
        Line::from(Span::raw("<n>g: Go to line <n>")),
        Line::from(Span::raw("e: Go to last line")),
    ];
    let navigation_paragraph = Paragraph::new(navigation_text)
        .block(navigation_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(navigation_paragraph, area);

    let cursor_x = area.x + app.navigation_input.len() as u16 + 13;
    let cursor_y = area.y + 1;
    frame.set_cursor(cursor_x, cursor_y);
}

fn render_help_overlay(frame: &mut Frame, size: Rect) {
    let help_area = centered_rect(50, 50, size);
    let help_block = Block::default()
        .borders(Borders::ALL)
        .title("Help - Key Bindings");

    let help_text = vec![
        Line::from(Span::raw("q: Quit")),
        Line::from(Span::raw("a: Add Task")),
        Line::from(Span::raw("A: Add Subtask")),
        Line::from(Span::raw("v: View Mode")),
        Line::from(Span::raw("f: Add Filter Criterion")),
        Line::from(Span::raw("c: Toggle Task Completion")),
        Line::from(Span::raw("k: Navigate Up")),
        Line::from(Span::raw("j: Navigate Down")),
        Line::from(Span::raw("p: Debug Overlay")),
        Line::from(Span::raw("g: Navigation Mode")),
        Line::from(Span::raw("C: Calendar Mode")),
        Line::from(Span::raw("?: Show Help")),
        Line::from(Span::raw("Esc: Return to Normal Mode")),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(help_block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(help_paragraph, help_area);
}

fn render_debug_overlay(frame: &mut Frame, size: Rect) {
    let help_area = centered_rect(50, 50, size);
    let help_block = Block::default()
        .borders(Borders::ALL)
        .title("Help - Key Bindings");

    let help_text = vec![
        Line::from(Span::raw("q: Quit")),
        Line::from(Span::raw("a: Add Task")),
        Line::from(Span::raw("A: Add Subtask")),
        Line::from(Span::raw("v: View Mode")),
        Line::from(Span::raw("f: Add Filter Criterion")),
        Line::from(Span::raw("c: Toggle Task Completion")),
        Line::from(Span::raw("k: Navigate Up")),
        Line::from(Span::raw("j: Navigate Down")),
        Line::from(Span::raw("p: Debug Overlay")),
        Line::from(Span::raw("g: Navigation Mode")),
        Line::from(Span::raw("C: Calendar Mode")),
        Line::from(Span::raw("?: Show Help")),
        Line::from(Span::raw("Esc: Return to Normal Mode")),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(help_block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(help_paragraph, help_area);
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

    if let Some(start_time) = task.start_time {
        description_spans.push(Span::styled(
            format!("[Start: {}]", start_time.format("%Y-%m-%d %H:%M")),
            Style::default().fg(Color::Blue),
        ));
    }

    if let Some(due_time) = task.due_time {
        description_spans.push(Span::styled(
            format!("[Due: {}]", due_time.format("%Y-%m-%d %H:%M")),
            Style::default().fg(Color::Red),
        ));
    }

    let total_subtasks = task.subtasks.len();
    if total_subtasks > 0 {
        let completed_subtasks = task.subtasks.values().filter(|t| t.completed).count();
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

fn render_calendar_mode(frame: &mut Frame, app: &Model, size: Rect) {
    let calendar_area = centered_rect(80, 80, size);
    let calendar_block = Block::default()
        .borders(Borders::ALL)
        .title("Calendar View");
    frame.render_widget(calendar_block, calendar_area);

    // Call the render_calendar function we defined earlier
    render_calendar(frame, app, calendar_area);
}

fn render_calendar(frame: &mut Frame, app: &Model, area: Rect) {
    let now = chrono::Local::now();
    let (year, month, today) = (now.year(), now.month(), now.day());
    let days_in_month = days_in_month(year, month);

    let calendar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let header =
        Paragraph::new(format!("{} {}", month_name(month), year)).alignment(Alignment::Center);
    frame.render_widget(header, calendar_layout[0]);

    let calendar_area = calendar_layout[1];
    let day_width = calendar_area.width / 7;
    let day_height = calendar_area.height / 6;

    for week in 0..6 {
        for day in 0..7 {
            let day_number = week * 7 + day + 1;
            if day_number <= days_in_month {
                let day_area = Rect::new(
                    calendar_area.x + (day as u16) * day_width,
                    calendar_area.y + (week as u16) * day_height,
                    day_width,
                    day_height,
                );

                let mut style = Style::default();
                if day_number == today {
                    style = style.bg(Color::Blue);
                }

                let day_block = Block::default().borders(Borders::ALL).style(style);
                frame.render_widget(day_block, day_area);

                let day_text = Paragraph::new(day_number.to_string()).alignment(Alignment::Center);
                frame.render_widget(day_text, day_area);

                // Here, you would render tasks for this day
                // You'll need to implement a function to get tasks for a specific day
                render_tasks_for_day(frame, app, day_area, year, month, day_number);
            }
        }
    }
}

fn render_tasks_for_day(
    frame: &mut Frame,
    app: &Model,
    area: Rect,
    year: i32,
    month: u32,
    day: u32,
) {
    let tasks_for_day = app.tasks.values().filter(|task| {
        if let Some(start_time) = task.start_time {
            start_time.year() == year && start_time.month() == month && start_time.day() == day
        } else {
            false
        }
    });

    let task_area = Rect::new(area.x + 1, area.y + 2, area.width - 2, area.height - 3);
    let task_list: Vec<ListItem> = tasks_for_day
        .take((task_area.height as usize).saturating_sub(1))
        .map(|task| {
            ListItem::new(Span::styled(
                task.description
                    .chars()
                    .take(task_area.width as usize)
                    .collect::<String>(),
                Style::default().fg(Color::Yellow),
            ))
        })
        .collect();

    let tasks_list = List::new(task_list);
    frame.render_widget(tasks_list, task_area);
}

fn days_in_month(year: i32, month: u32) -> u32 {
    chrono::NaiveDate::from_ymd_opt(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .unwrap()
    .signed_duration_since(chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days() as u32
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => unreachable!(),
    }
}
