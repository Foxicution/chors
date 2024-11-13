use crate::model::Task;
use ratatui::{
    style::{Color, Style},
    text::Span,
};

pub fn style_task(task: &Task, ident: usize) -> Vec<Span> {
    let ident = "  ".repeat(ident);
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
    }

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

    description_spans
}

pub fn style_input_task(input: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '#' => {
                // Style words starting with #
                let mut tag = String::new();
                tag.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() {
                        break;
                    }
                    tag.push(chars.next().unwrap());
                }
                spans.push(Span::styled(tag, Style::default().fg(Color::Magenta)));
            }
            '@' => {
                // Style words starting with @
                let mut context = String::new();
                context.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() {
                        break;
                    }
                    context.push(chars.next().unwrap());
                }
                spans.push(Span::styled(context, Style::default().fg(Color::Cyan)));
            }
            _ => {
                // Add other text as raw, without styling
                let mut word = c.to_string();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() || "#@".contains(next_c) {
                        break;
                    }
                    word.push(chars.next().unwrap());
                }
                spans.push(Span::raw(word));
            }
        }

        // Preserve whitespace as raw spans
        if c.is_whitespace() {
            spans.push(Span::raw(c.to_string()));
        }
    }

    spans
}

pub fn style_input_filter(input: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Brackets keep the default style
            '(' | ')' => {
                spans.push(Span::raw(c.to_string()));
            }
            '"' => {
                // Strings denoted by ""
                let mut string_content = String::new();
                string_content.push(c);
                while let Some(&next_c) = chars.peek() {
                    string_content.push(chars.next().unwrap());
                    if next_c == '"' {
                        break;
                    }
                }
                spans.push(Span::styled(
                    string_content,
                    Style::default().fg(Color::Green),
                ));
            }
            '#' => {
                // Words starting with #
                let mut word = String::new();
                word.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() {
                        break;
                    }
                    word.push(chars.next().unwrap());
                }
                spans.push(Span::styled(word, Style::default().fg(Color::Magenta)));
            }
            '@' => {
                // Words starting with @
                let mut word = String::new();
                word.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() {
                        break;
                    }
                    word.push(chars.next().unwrap());
                }
                spans.push(Span::styled(word, Style::default().fg(Color::Cyan)));
            }
            '[' => {
                // Constructs [ ] and [x]
                let mut construct = String::new();
                construct.push(c);
                if let Some(&next_c) = chars.peek() {
                    if next_c == 'x' || next_c == ' ' {
                        construct.push(chars.next().unwrap());
                        if let Some(&']') = chars.peek() {
                            construct.push(chars.next().unwrap());
                            let style = match construct.as_str() {
                                "[x]" => Style::default().fg(Color::Green),
                                "[ ]" => Style::default().fg(Color::Yellow),
                                _ => Style::default(),
                            };
                            spans.push(Span::styled(construct, style));
                        } else {
                            spans.push(Span::raw(construct));
                        }
                    } else {
                        spans.push(Span::raw(construct));
                    }
                } else {
                    spans.push(Span::raw(construct));
                }
            }
            _ => {
                if c.is_whitespace() {
                    spans.push(Span::raw(c.to_string()));
                } else {
                    // Words and keywords
                    let mut word = c.to_string();
                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_whitespace() || "()[]\"#@".contains(next_c) {
                            break;
                        }
                        word.push(chars.next().unwrap());
                    }
                    match word.as_str() {
                        "and" | "or" | "not" => {
                            spans.push(Span::styled(word, Style::default().fg(Color::Blue)));
                        }
                        _ => {
                            spans.push(Span::raw(word));
                        }
                    }
                }
            }
        }
    }

    spans
}
