use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take_while1},
    character::complete::{alphanumeric1, anychar, char, multispace0, multispace1, one_of},
    combinator::{all_consuming, map, opt, recognize},
    error::{context, VerboseError},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded},
    IResult,
};

/// Represents different types of tokens parsed from input.
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Tag(&'a str),
    Context(&'a str),
    Text(&'a str),
    NotOperator,
    AndOperator,
    OrOperator,
    Completed,
    Incomplete,
    QuotedText(&'a str),
    Parenthesis(char),
    Word(&'a str),
    Whitespace(&'a str),
    Symbol(char),
    // Add more tokens as needed
}

// Parsing functions

/// Parse a tag (e.g., `#tagname`).
pub fn parse_tag(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context(
        "tag",
        map(
            recognize(pair(
                char('#'),
                take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
            )),
            |tag_text: &str| Token::Tag(&tag_text[1..]), // Skip the '#' character
        ),
    )(input)
}

/// Parse a context (e.g., `@contextname`).
pub fn parse_context(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context(
        "context",
        map(
            recognize(pair(
                char('@'),
                take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
            )),
            |context_text: &str| Token::Context(&context_text[1..]), // Skip the '@' character
        ),
    )(input)
}

/// Parse a quoted text (e.g., `"some text"`).
pub fn parse_quoted_text(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context(
        "quoted text",
        map(
            recognize(delimited(char('"'), is_not("\""), opt(char('"')))),
            |quoted_text: &str| Token::QuotedText(quoted_text),
        ),
    )(input)
}

/// Parse the 'not' operator.
pub fn parse_not_operator(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context(
        "not operator",
        map(tag_no_case("not"), |_| Token::NotOperator),
    )(input)
}

/// Parse the 'and' operator.
pub fn parse_and_operator(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context(
        "and operator",
        map(tag_no_case("and"), |_| Token::AndOperator),
    )(input)
}

/// Parse the 'or' operator.
pub fn parse_or_operator(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context("or operator", map(tag_no_case("or"), |_| Token::OrOperator))(input)
}

/// Parse the completed marker `[x]`.
pub fn parse_completed(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context("completed", map(tag("[x]"), |_| Token::Completed))(input)
}

/// Parse the incomplete marker `[ ]`.
pub fn parse_incomplete(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context("incomplete", map(tag("[ ]"), |_| Token::Incomplete))(input)
}

/// Parse a parenthesis.
pub fn parse_parenthesis(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context(
        "parenthesis",
        map(one_of("()"), |paren| Token::Parenthesis(paren)),
    )(input)
}

/// Parse a word (any sequence of non-special characters).
pub fn parse_word(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context(
        "word",
        map(
            take_while1(|c: char| {
                !c.is_whitespace()
                    && c != '#'
                    && c != '@'
                    && c != '('
                    && c != ')'
                    && c != '['
                    && c != ']'
                    && c != '"'
            }),
            |word: &str| Token::Word(word),
        ),
    )(input)
}

/// Parse whitespace.
pub fn parse_whitespace(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context(
        "whitespace",
        map(multispace1, |ws: &str| Token::Whitespace(ws)),
    )(input)
}

/// Parse any symbol (single character not matched by other parsers).
pub fn parse_symbol(input: &str) -> IResult<&str, Token, VerboseError<&str>> {
    context("symbol", map(anychar, |c| Token::Symbol(c)))(input)
}

/// Helper function to parse a sequence of tokens.
pub fn parse_tokens(input: &str) -> IResult<&str, Vec<Token>, VerboseError<&str>> {
    many0(alt((
        parse_tag,
        parse_context,
        parse_not_operator,
        parse_and_operator,
        parse_or_operator,
        parse_completed,
        parse_incomplete,
        parse_quoted_text,
        parse_parenthesis,
        parse_word,
        parse_whitespace,
        parse_symbol,
    )))(input)
}
