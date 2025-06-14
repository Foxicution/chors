use crate::{
    model::{DisplayMessage, Mode, Model, Overlay},
    tui::{
        run::KEYBINDS,
        style::{style_input_filter, style_input_task, style_task},
    },
    utils::{VectorUtils, key_to_char, mod_to_str},
};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListDirection, ListItem, ListState, Paragraph, Wrap},
};
use std::io;

type Tui = Terminal<CrosstermBackend<io::Stdout>>;

const INFO_HEIGHT: u16 = 1;
const INPUT_HEIGHT: u16 = 1;
const STATUS_HEIGHT: u16 = INFO_HEIGHT + INPUT_HEIGHT;

pub fn ui(frame: &mut Frame, model: &Model) {
    let size = frame.size();
    let available_height = size.height.saturating_sub(STATUS_HEIGHT);
    let available_size = Rect { height: available_height, ..size };

    match model.mode {
        Mode::List => render_mode_list(frame, model, available_size),
    }

    match model.overlay {
        Overlay::Help => render_help_overlay(frame, model, available_size),
        Overlay::SelectingFilter => render_overlay_selectingfilter(frame, model, available_size),
        _ => {}
    }

    render_taskbar(frame, model, size);
}

fn render_help_overlay(frame: &mut Frame, model: &Model, area: Rect) {
    // 1) Gather only the bindings for this mode
    let entries: Vec<_> = KEYBINDS.iter().filter(|kb| kb.modes.contains(&model.mode)).collect();
    if entries.is_empty() {
        panic!("All modes should have at least one keybind...")
    }

    // 2) Compute column widths based on the longest key‐string and description
    let key_max = entries
        .iter()
        .map(|kb| {
            kb.keys
                .iter()
                .map(|(k, m)| {
                    if m.is_empty() {
                        format!("{}", key_to_char(k))
                    } else {
                        format!("{}-{}", mod_to_str(m), key_to_char(k))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
                .len()
        })
        .max()
        .unwrap_or(0);

    let desc_max = entries.iter().map(|kb| kb.description.len()).max().unwrap_or(0);

    // 3) Total popup size: key column + space + desc column + padding + border (2×1)
    let padding = 2; // one space each side
    let border = 2; // left + right
    let width = ((key_max + 3 + desc_max + padding + border) as u16).min(area.width); // never exceed container

    let height = (entries.len() as u16) + 2; // top/bottom border

    // 4) Anchor bottom-right inside `area`
    let x = area.x + area.width.saturating_sub(width);
    let y = area.y + area.height.saturating_sub(height);
    let area = Rect::new(x, y, width, height);

    // 5) Build and render the list
    let items: Vec<ListItem> = entries
        .into_iter()
        .map(|kb| {
            let keystr = kb
                .keys
                .iter()
                .map(|(k, m)| {
                    if m.is_empty() {
                        format!("{}", key_to_char(k))
                    } else {
                        format!("{}+{}", mod_to_str(m), key_to_char(k))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            // pad keystr to key_max, then a space, then description
            let line = Line::from(vec![
                Span::raw(format!("{:<key_max$}", keystr, key_max = key_max)),
                Span::raw("   "),
                Span::raw(kb.description),
            ]);
            ListItem::new(line)
        })
        .collect();

    let help = List::new(items)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Indexed(237))),
        )
        .direction(ListDirection::TopToBottom);

    frame.render_widget(help, area);
}

fn render_overlay_selectingfilter(frame: &mut Frame, model: &Model, area: Rect) {
    let popup_width = (area.width * 80 / 100).min(area.width * 4 / 5);
    let popup_height = (area.height * 70 / 100).min(area.height * 2 / 3);

    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("[ Select Filter (Type to search, ↑↓ to navigate, Enter to select, Esc to close) ]")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    frame.render_widget(block.clone(), popup_area);

    let inner_area = block.inner(popup_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner_area);

    frame.set_cursor(chunks[0].x + model.input.cursor as u16, chunks[0].y);
    frame.render_widget(model.input.text.clone(), chunks[0]);

    frame.render_widget("─".repeat(chunks[1].width as usize), chunks[1]);

    let items: Vec<ListItem> =
        model.filters.iter().map(|(_, filter)| ListItem::new(filter.name.clone())).collect();

    let list = List::new(items).highlight_style(Style::default().bg(Color::Indexed(238)));
    let mut list_state = ListState::default().with_selected(Some(0));

    frame.render_stateful_widget(list, chunks[2], &mut list_state);
}

fn render_mode_list(frame: &mut Frame, model: &Model, area: Rect) {
    if model.filtered_tasks.is_empty() {
        if let Overlay::AddingSiblingTask = model.overlay {
            let selected_index = Some(0);
            let (ident, new_task) = new_task_list_item(model);

            let list =
                List::new(vec![new_task]).highlight_style(Style::new().bg(Color::Indexed(238)));
            let mut selection_state = ListState::default().with_selected(selected_index);

            let cursor_x = 4 + 2 * ident + 4 + model.input.cursor as u16;
            let cursor_y = area.y;
            frame.set_cursor(cursor_x, cursor_y);

            frame.render_stateful_widget(list, area, &mut selection_state);
        } else {
            // Step 1: Calculate the dimensions for a centered message
            let message = "Wow so Empty!\nAdd some todos to get started!";
            let message_height = 2; // Adjust this to match the number of lines in the message
            let y_offset = (area.height / 2).saturating_sub(message_height); // Center vertically
            let x_offset = area.width / 4; // Adjust for a bit of padding on the sides

            // Step 2: Define a centered rectangle for the message
            let centered_rect = Rect::new(
                x_offset,
                y_offset,
                area.width / 2,     // Make the message take half the screen width
                message_height * 2, // Approximate height for the message box
            );

            // Step 3: Create and style the paragraph for the message
            let empty_message = Paragraph::new(message)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));

            // Render the paragraph at the calculated position
            frame.render_widget(empty_message, centered_rect);
        }
    } else {
        let mut selected_task_index = match model.selected_task {
            Some(id) => model.filtered_tasks.get_index(&id),
            None => None,
        };

        let mut task_list: Vec<_> = model
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
            })
            .collect();

        if let Overlay::AddingSiblingTask | Overlay::AddingChildTask = model.overlay {
            let selected_task_children = match model.selected_task {
                Some(id) => {
                    let selected_path = model.filtered_tasks.get(&id).unwrap();
                    let prefix_len = selected_path.len();

                    model
                        .filtered_tasks
                        .iter()
                        .filter(|(_id, path)| {
                            path.len() > prefix_len
                                && path
                                    .iter()
                                    .take(prefix_len)
                                    .zip(selected_path.iter())
                                    .all(|(a, b)| a == b)
                        })
                        .count()
                }
                None => 0,
            };
            let insertion_index = selected_task_index
                .map(|idx| {
                    if let Overlay::AddingSiblingTask = model.overlay {
                        idx + selected_task_children + 1
                    } else {
                        idx + 1
                    }
                })
                .unwrap_or(0);
            selected_task_index = Some(insertion_index);

            let (ident, new_task) = new_task_list_item(model);
            task_list.insert(insertion_index, new_task);

            let cursor_x = 4 + 2 * ident + 4 + model.input.cursor as u16;
            let cursor_y = area.y + insertion_index as u16;
            frame.set_cursor(cursor_x, cursor_y);
        }

        let list = List::new(task_list).highlight_style(Style::default().bg(Color::Indexed(238)));
        let mut selection_state = ListState::default().with_selected(selected_task_index);

        frame.render_stateful_widget(list, area, &mut selection_state);
    }
}

fn render_taskbar(frame: &mut Frame, model: &Model, area: Rect) {
    let info_area = Rect::new(area.x, area.height - STATUS_HEIGHT, area.width, INFO_HEIGHT);
    let input_area = Rect::new(area.x, area.height - INPUT_HEIGHT, area.width, INPUT_HEIGHT);

    let info_paragraph = Paragraph::new(format!(
        " <{}>",
        model.get_selected_filter().map(|f| f.name.as_str()).unwrap_or("")
    ))
    .bg(Color::Indexed(239));

    let input_paragraph = match model.overlay {
        Overlay::None
        | Overlay::SelectingFilter
        | Overlay::AddingSiblingTask
        | Overlay::AddingChildTask
        | Overlay::Help => match model.message.clone() {
            DisplayMessage::None => Paragraph::new(""),
            DisplayMessage::Success(msg) => Paragraph::new(msg).fg(Color::LightGreen),
            DisplayMessage::Error(msg) => Paragraph::new(msg).fg(Color::LightRed),
        },
        // Old behavior, moved to the task view, might add back in calendar mode
        // Overlay::AddingSiblingTask | Overlay::AddingChildTask => {
        //     frame.set_cursor(model.input.cursor as u16, input_area.y);
        //     Paragraph::new(Line::from(style_input_task(&model.input.text)))
        // }
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

fn new_task_list_item(model: &Model) -> (u16, ListItem) {
    let ident = model
        .selected_task
        .map(|id| model.filtered_tasks.get(&id))
        .map(|path| path.unwrap().len())
        .map(|len| if let Overlay::AddingChildTask = model.overlay { len + 1 } else { len })
        .unwrap_or(0);

    let mut line_spans = vec![
        Span::raw("    "),
        Span::raw("  ".repeat(ident)),
        Span::styled("[ ] ", Style::default().fg(Color::LightYellow)),
    ];
    let input_spans = style_input_task(&model.input.text).into_iter().map(|span| {
        if span.style == Style::default() {
            Span::styled(
                span.content,
                Style::default().fg(Color::LightGreen).add_modifier(Modifier::ITALIC),
            )
        } else {
            Span::styled(span.content, span.style.add_modifier(Modifier::ITALIC))
        }
    });
    line_spans.extend(input_spans);
    let line_item = ListItem::new(Line::from(line_spans));
    (ident as u16, line_item)
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
