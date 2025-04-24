use crate::{
    model::Task,
    parse::{
        Token, parse_and_operator, parse_completed, parse_context, parse_incomplete,
        parse_not_operator, parse_or_operator, parse_parenthesis, parse_quoted_text, parse_symbol,
        parse_tag, parse_tokens, parse_whitespace, parse_word,
    },
};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take_while1},
    character::complete::{alphanumeric1, anychar, char, multispace1, one_of},
    combinator::{map, opt, recognize},
    multi::many0,
    sequence::{delimited, preceded},
};
use ratatui::{
    style::{Color, Style},
    text::Span,
};

/// Style a task for display.
pub fn style_task(task: &Task, ident: usize) -> Vec<Span> {
    let ident = "  ".repeat(ident); // Adds indentation based on the specified level.
    let status = if task.completed.is_some() {
        Span::styled("[x]", Style::default().fg(Color::LightGreen))
    } else {
        Span::styled("[ ]", Style::default().fg(Color::LightYellow))
    };

    // Start building the list of spans with indentation and status
    let mut description_spans = vec![Span::raw(ident), status, Span::raw(" ")];

    // Parse and style the task description
    match parse_tokens(&task.description) {
        Ok((_, tokens)) => {
            description_spans.extend(tokens.into_iter().map(token_to_span));
        }
        Err(_) => {
            // If parsing fails, treat the entire description as raw text
            description_spans.push(Span::raw(task.description.to_string()));
        }
    }

    // Add subtasks completion count if there are subtasks
    if !task.subtasks.is_empty() {
        let total_subtasks = task.subtasks.len();
        let completed_subtasks = task
            .subtasks
            .iter()
            .filter(|(_, t)| t.completed.is_some())
            .count();

        let color = if completed_subtasks == total_subtasks {
            Color::LightGreen
        } else {
            Color::LightYellow
        };

        description_spans.push(Span::styled(
            format!(" [{}/{}]", completed_subtasks, total_subtasks),
            Style::default().fg(color),
        ));
    }

    description_spans
}

/// Style the input task description using `nom` parsers.
pub fn style_input_task(input: &str) -> Vec<Span> {
    match parse_tokens(input) {
        Ok((_, tokens)) => tokens.into_iter().map(token_to_span).collect(),
        Err(_) => vec![Span::raw(input.to_string())],
    }
}

fn token_to_span(token: Token) -> Span {
    match token {
        Token::Tag(tag) => Span::styled(
            format!("#{}", tag),
            Style::default().fg(Color::LightMagenta),
        ),
        Token::Context(context) => Span::styled(
            format!("@{}", context),
            Style::default().fg(Color::LightCyan),
        ),
        Token::NotOperator => Span::raw("not"),
        Token::AndOperator => Span::raw("and"),
        Token::OrOperator => Span::raw("or"),
        Token::Completed => Span::raw("[x]"),
        Token::Incomplete => Span::raw("[ ]"),
        Token::QuotedText(text) => Span::raw(text),
        Token::Parenthesis(c) => Span::raw(c.to_string()),
        Token::Word(word) => Span::raw(word.to_string()),
        Token::Whitespace(ws) => Span::raw(ws.to_string()),
        Token::Symbol(c) => Span::raw(c.to_string()),
    }
}

/// Style the input filter string using `nom` parsers.
pub fn style_input_filter(input: &str) -> Vec<Span> {
    match parse_tokens(input) {
        Ok((_, tokens)) => tokens.into_iter().map(token_to_filter_span).collect(),
        Err(_) => vec![Span::raw(input.to_string())],
    }
}

fn token_to_filter_span(token: Token) -> Span {
    match token {
        Token::Tag(tag) => Span::styled(
            format!("#{}", tag),
            Style::default().fg(Color::LightMagenta),
        ),
        Token::Context(context) => Span::styled(
            format!("@{}", context),
            Style::default().fg(Color::LightCyan),
        ),
        Token::NotOperator => Span::styled("not", Style::default().fg(Color::LightRed)),
        Token::AndOperator => Span::styled("and", Style::default().fg(Color::LightBlue)),
        Token::OrOperator => Span::styled("or", Style::default().fg(Color::LightBlue)),
        Token::Completed => Span::styled("[x]", Style::default().fg(Color::LightGreen)),
        Token::Incomplete => Span::styled("[ ]", Style::default().fg(Color::LightYellow)),
        Token::QuotedText(text) => {
            Span::styled(text.to_string(), Style::default().fg(Color::LightGreen))
        }
        Token::Parenthesis(c) => Span::raw(c.to_string()),
        Token::Word(word) => Span::raw(word.to_string()),
        Token::Whitespace(ws) => Span::raw(ws.to_string()),
        Token::Symbol(c) => Span::raw(c.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use ratatui::style::{Color, Style};
    use ratatui::text::Span;

    #[test]
    fn test_style_input_task() {
        let input = "Complete the report #work @office";
        let spans = style_input_task(input);

        assert_eq!(
            spans,
            vec![
                Span::raw("Complete"),
                Span::raw(" "),
                Span::raw("the"),
                Span::raw(" "),
                Span::raw("report"),
                Span::raw(" "),
                Span::styled("#work", Style::default().fg(Color::LightMagenta)),
                Span::raw(" "),
                Span::styled("@office", Style::default().fg(Color::LightCyan)),
            ]
        );
    }

    #[test]
    fn test_style_input_task_with_empty_tag() {
        let input = "Finish the task # @home";
        let spans = style_input_task(input);

        assert_eq!(
            spans,
            vec![
                Span::raw("Finish"),
                Span::raw(" "),
                Span::raw("the"),
                Span::raw(" "),
                Span::raw("task"),
                Span::raw(" "),
                Span::raw("#"),
                Span::raw(" "),
                Span::styled("@home", Style::default().fg(Color::LightCyan)),
            ]
        );
    }

    #[test]
    fn test_style_input_filter() {
        let input = "(not [x] and #work) or @office";
        let spans = style_input_filter(input);

        assert_eq!(
            spans,
            vec![
                Span::raw("("),
                Span::styled("not", Style::default().fg(Color::LightRed)),
                Span::raw(" "),
                Span::styled("[x]", Style::default().fg(Color::LightGreen)),
                Span::raw(" "),
                Span::styled("and", Style::default().fg(Color::LightBlue)),
                Span::raw(" "),
                Span::styled("#work", Style::default().fg(Color::LightMagenta)),
                Span::raw(")"),
                Span::raw(" "),
                Span::styled("or", Style::default().fg(Color::LightBlue)),
                Span::raw(" "),
                Span::styled("@office", Style::default().fg(Color::LightCyan)),
            ]
        );
    }

    #[test]
    fn test_style_input_filter_with_quotes() {
        let input = "(not \"meeting notes\" and [ ] or @home)";
        let spans = style_input_filter(input);

        assert_eq!(
            spans,
            vec![
                Span::raw("("),
                Span::styled("not", Style::default().fg(Color::LightRed)),
                Span::raw(" "),
                Span::styled("\"meeting notes\"", Style::default().fg(Color::LightGreen)),
                Span::raw(" "),
                Span::styled("and", Style::default().fg(Color::LightBlue)),
                Span::raw(" "),
                Span::styled("[ ]", Style::default().fg(Color::LightYellow)),
                Span::raw(" "),
                Span::styled("or", Style::default().fg(Color::LightBlue)),
                Span::raw(" "),
                Span::styled("@home", Style::default().fg(Color::LightCyan)),
                Span::raw(")"),
            ]
        );
    }

    #[test]
    fn test_style_input_filter_with_empty_tag() {
        let input = "# and @";
        let spans = style_input_filter(input);

        assert_eq!(
            spans,
            vec![
                Span::raw("#"),
                Span::raw(" "),
                Span::styled("and", Style::default().fg(Color::LightBlue)),
                Span::raw(" "),
                Span::raw("@"),
            ]
        );
    }

    #[test]
    fn test_style_input_task_with_symbols() {
        let input = "Fix issue #123!";
        let spans = style_input_task(input);

        assert_eq!(
            spans,
            vec![
                Span::raw("Fix"),
                Span::raw(" "),
                Span::raw("issue"),
                Span::raw(" "),
                Span::styled("#123", Style::default().fg(Color::LightMagenta)),
                Span::raw("!"),
            ]
        );
    }
}
