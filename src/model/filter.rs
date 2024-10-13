use crate::model::task::Task;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case},
    character::complete::{alphanumeric1, char, multispace0},
    combinator::{all_consuming, map, recognize},
    error::{context, convert_error, VerboseError},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded},
    IResult,
};
use uuid::{NoContext, Timestamp, Uuid};

// Condition structs and implementations

#[derive(Clone)]
pub struct Tag {
    tag: String,
}

impl Tag {
    pub fn new(tag: String) -> Self {
        Tag { tag }
    }

    pub fn evaluate(&self, task: &Task) -> bool {
        task.tags.contains(&self.tag)
    }
}

#[derive(Clone)]
pub struct Context {
    context: String,
}

impl Context {
    pub fn new(context: String) -> Self {
        Context { context }
    }

    pub fn evaluate(&self, task: &Task) -> bool {
        task.contexts.contains(&self.context)
    }
}

#[derive(Clone)]
pub struct Completion {
    completed: bool,
}

impl Completion {
    pub fn new(completed: bool) -> Self {
        Completion { completed }
    }

    pub fn evaluate(&self, task: &Task) -> bool {
        task.completed.is_some() == self.completed
    }
}

#[derive(Clone)]
pub struct Text {
    text: String,
}

impl Text {
    pub fn new(text: String) -> Self {
        Text { text }
    }

    pub fn evaluate(&self, task: &Task) -> bool {
        task.description
            .to_lowercase()
            .contains(&self.text.to_lowercase())
    }
}

// Condition enum

#[derive(Clone)]
pub enum Condition {
    Tag(Tag),
    Context(Context),
    Completion(Completion),
    Text(Text),
    Not(Box<Condition>),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    AlwaysTrue,
}

impl Condition {
    pub fn evaluate(&self, task: &Task) -> bool {
        match self {
            Condition::Tag(cond) => cond.evaluate(task),
            Condition::Context(cond) => cond.evaluate(task),
            Condition::Completion(cond) => cond.evaluate(task),
            Condition::Text(cond) => cond.evaluate(task),
            Condition::Not(cond) => !cond.evaluate(task),
            Condition::And(left, right) => left.evaluate(task) && right.evaluate(task),
            Condition::Or(left, right) => left.evaluate(task) || right.evaluate(task),
            Condition::AlwaysTrue => true,
        }
    }
}

// Filter structs

#[derive(Clone)]
pub struct FilterCondition {
    pub expression: String,
    pub condition: Condition,
}

impl FilterCondition {
    pub fn new(expression: String) -> Result<Self, String> {
        if expression.trim().is_empty() {
            Ok(FilterCondition {
                expression,
                condition: Condition::AlwaysTrue,
            })
        } else {
            let condition = parse_filter_expression(&expression)?;
            Ok(FilterCondition {
                expression,
                condition,
            })
        }
    }
}

#[derive(Clone)]
pub struct Filter {
    pub id: Uuid,
    pub name: String,
    pub filter_condition: FilterCondition,
}

impl Filter {
    pub fn new(name: String, filter_condition: FilterCondition) -> Self {
        Filter {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            name,
            filter_condition,
        }
    }
}

// Parsing functions

pub fn parse_filter_expression(input: &str) -> Result<Condition, String> {
    match all_consuming(expression)(input) {
        Ok((_, condition)) => Ok(condition),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            // Convert the nom error to a human-readable error message
            let err_msg = convert_error(input, e);
            Err(format!("Parsing error: {}", err_msg))
        }
        Err(nom::Err::Incomplete(_)) => Err("Incomplete input provided".to_string()),
    }
}

fn expression(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "expression",
        map(
            pair(
                term,
                many0(pair(
                    preceded(multispace0, tag_no_case("or")),
                    preceded(multispace0, term),
                )),
            ),
            |(first, rest)| {
                rest.into_iter().fold(first, |acc, (_, term)| {
                    Condition::Or(Box::new(acc), Box::new(term))
                })
            },
        ),
    )(input)
}

fn term(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "term",
        map(
            pair(
                factor,
                many0(pair(
                    preceded(multispace0, tag_no_case("and")),
                    preceded(multispace0, factor),
                )),
            ),
            |(first, rest)| {
                rest.into_iter().fold(first, |acc, (_, factor)| {
                    Condition::And(Box::new(acc), Box::new(factor))
                })
            },
        ),
    )(input)
}

fn factor(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "factor",
        alt((
            map(
                preceded(pair(tag_no_case("not"), multispace0), factor),
                |cond| Condition::Not(Box::new(cond)),
            ),
            operand,
        )),
    )(input)
}

fn operand(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "operand",
        alt((
            map(delimited(char('('), expression, char(')')), |cond| cond),
            completed,
            incomplete,
            tag_condition,
            context_condition,
            text_condition,
        )),
    )(input)
}

fn completed(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "completed condition",
        map(tag("[x]"), |_| Condition::Completion(Completion::new(true))),
    )(input)
}

fn incomplete(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "incomplete condition",
        map(tag("[ ]"), |_| {
            Condition::Completion(Completion::new(false))
        }),
    )(input)
}

fn tag_condition(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "tag condition",
        map(preceded(char('#'), identifier), |tag_name: &str| {
            Condition::Tag(Tag::new(tag_name.to_string()))
        }),
    )(input)
}

fn context_condition(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "context condition",
        map(preceded(char('@'), identifier), |context_name: &str| {
            Condition::Context(Context::new(context_name.to_string()))
        }),
    )(input)
}

fn text_condition(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "text condition",
        map(
            delimited(char('"'), is_not("\""), char('"')),
            |text: &str| Condition::Text(Text::new(text.to_string())),
        ),
    )(input)
}

fn identifier(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    recognize(many1(alt((alphanumeric1, tag("."), tag("_"), tag("-")))))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::task::Task, utils::reorderable_map::ReorderableMap};
    use chrono::Utc;
    use uuid::{NoContext, Timestamp, Uuid};

    fn create_test_task() -> Task {
        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: "Complete the urgent report #work @office".to_string(),
            tags: vec!["work".to_string()].into_iter().collect(),
            contexts: vec!["office".to_string()].into_iter().collect(),
            completed: None,
            subtasks: ReorderableMap::new(),
        }
    }

    fn create_completed_task() -> Task {
        Task {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            description: "Buy groceries #errand @shopping".to_string(),
            tags: vec!["errand".to_string()].into_iter().collect(),
            contexts: vec!["shopping".to_string()].into_iter().collect(),
            completed: Some(Utc::now()),
            subtasks: ReorderableMap::new(),
        }
    }

    #[test]
    fn test_tag_condition() {
        let task = create_test_task();
        let condition = Tag::new("work".to_string());
        assert!(condition.evaluate(&task));

        let wrong_tag_condition = Tag::new("personal".to_string());
        assert!(!wrong_tag_condition.evaluate(&task));
    }

    #[test]
    fn test_context_condition() {
        let task = create_test_task();
        let condition = Context::new("office".to_string());
        assert!(condition.evaluate(&task));

        let wrong_context_condition = Context::new("home".to_string());
        assert!(!wrong_context_condition.evaluate(&task));
    }

    #[test]
    fn test_completion_condition() {
        let task = create_test_task();
        let completed_task = create_completed_task();

        let condition_completed = Completion::new(true);
        assert!(!condition_completed.evaluate(&task));
        assert!(condition_completed.evaluate(&completed_task));

        let condition_incomplete = Completion::new(false);
        assert!(condition_incomplete.evaluate(&task));
        assert!(!condition_incomplete.evaluate(&completed_task));
    }

    #[test]
    fn test_text_condition() {
        let task = create_test_task();
        let condition = Text::new("urgent".to_string());
        assert!(condition.evaluate(&task));

        let wrong_text_condition = Text::new("birthday".to_string());
        assert!(!wrong_text_condition.evaluate(&task));
    }

    #[test]
    fn test_not_condition() {
        let task = create_test_task();
        let tag_condition = Tag::new("work".to_string());
        let not_condition = Condition::Not(Box::new(Condition::Tag(tag_condition)));

        assert!(!not_condition.evaluate(&task));
    }

    #[test]
    fn test_and_condition() {
        let task = create_test_task();
        let tag_condition = Tag::new("work".to_string());
        let context_condition = Context::new("office".to_string());

        let and_condition = Condition::And(
            Box::new(Condition::Tag(tag_condition)),
            Box::new(Condition::Context(context_condition)),
        );
        assert!(and_condition.evaluate(&task));

        let wrong_and_condition = Condition::And(
            Box::new(Condition::Tag(Tag::new("personal".to_string()))),
            Box::new(Condition::Context(Context::new("office".to_string()))),
        );
        assert!(!wrong_and_condition.evaluate(&task));
    }

    #[test]
    fn test_or_condition() {
        let task = create_test_task();
        let tag_condition = Tag::new("work".to_string());
        let wrong_context_condition = Context::new("home".to_string());

        let or_condition = Condition::Or(
            Box::new(Condition::Tag(tag_condition)),
            Box::new(Condition::Context(wrong_context_condition)),
        );
        assert!(or_condition.evaluate(&task));

        let wrong_or_condition = Condition::Or(
            Box::new(Condition::Tag(Tag::new("personal".to_string()))),
            Box::new(Condition::Context(Context::new("home".to_string()))),
        );
        assert!(!wrong_or_condition.evaluate(&task));
    }

    #[test]
    fn test_always_true_condition() {
        let task = create_test_task();
        let condition = Condition::AlwaysTrue;
        assert!(condition.evaluate(&task));
    }

    #[test]
    fn test_parse_simple_expression() {
        let task = create_test_task();
        let expression = "not [x] and #work";
        let filter_condition = parse_filter_expression(expression).unwrap();
        assert!(filter_condition.evaluate(&task));

        let expression = "not [x] and #personal";
        let filter_condition = parse_filter_expression(expression).unwrap();
        assert!(!filter_condition.evaluate(&task));
    }

    #[test]
    fn test_parse_complex_expression() {
        let task = create_test_task();
        let completed_task = create_completed_task();

        let expression = "#work or @home";
        let filter_condition = parse_filter_expression(expression).unwrap();
        assert!(filter_condition.evaluate(&task));
        assert!(!filter_condition.evaluate(&completed_task));

        let expression = "[x] or #work";
        let filter_condition = parse_filter_expression(expression).unwrap();
        assert!(filter_condition.evaluate(&task));
        assert!(filter_condition.evaluate(&completed_task));
    }

    #[test]
    fn test_parse_expression_with_parentheses() {
        let task = create_test_task();
        let expression = "(not [x]) and (@office or #work)";
        let filter_condition = parse_filter_expression(expression).unwrap();
        assert!(filter_condition.evaluate(&task));
    }

    #[test]
    fn test_parse_invalid_expression() {
        let expression = "not #";
        let result = parse_filter_expression(expression);
        assert!(result.is_err());

        let expression = "[x] and";
        let result = parse_filter_expression(expression);
        assert!(result.is_err());
    }
}
