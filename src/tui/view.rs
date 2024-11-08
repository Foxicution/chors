use crate::model::Model;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
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

    render_taskbar(frame, model, size);
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
