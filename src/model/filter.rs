use crate::{
    model::task::Task,
    parse::{
        parse_and_operator, parse_completed, parse_context, parse_incomplete, parse_not_operator,
        parse_or_operator, parse_parenthesis, parse_quoted_text, parse_tag, Token,
    },
};
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
use std::rc::Rc;
use uuid::Uuid;

// Condition structs and implementations

#[derive(Clone, Debug)]
pub struct Tag {
    tag: Rc<String>,
}

impl Tag {
    pub fn new<S: Into<String>>(tag: S) -> Self {
        Tag {
            tag: tag.into().into(),
        }
    }

    pub fn evaluate(&self, task: &Task) -> bool {
        task.tags.contains(&*self.tag)
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    context: Rc<String>,
}

impl Context {
    pub fn new<S: Into<String>>(context: S) -> Self {
        Context {
            context: context.into().into(),
        }
    }

    pub fn evaluate(&self, task: &Task) -> bool {
        task.contexts.contains(&*self.context)
    }
}

impl PartialEq for Context {
    fn eq(&self, other: &Self) -> bool {
        self.context == other.context
    }
}

#[derive(Clone, Debug)]
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

impl PartialEq for Completion {
    fn eq(&self, other: &Self) -> bool {
        self.completed == other.completed
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    text: Rc<String>,
}

impl Text {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Text {
            text: text.into().into(),
        }
    }

    pub fn evaluate(&self, task: &Task) -> bool {
        task.description.contains(&*self.text)
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

// Condition enum

#[derive(Clone, Debug)]
pub enum Condition {
    Tag(Tag),
    Context(Context),
    Completion(Completion),
    Text(Text),
    Not(Rc<Condition>),
    And(Rc<Condition>, Rc<Condition>),
    Or(Rc<Condition>, Rc<Condition>),
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

impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Condition::Tag(tag1), Condition::Tag(tag2)) => tag1 == tag2,
            (Condition::Context(ctx1), Condition::Context(ctx2)) => ctx1 == ctx2,
            (Condition::Completion(comp1), Condition::Completion(comp2)) => comp1 == comp2,
            (Condition::Text(text1), Condition::Text(text2)) => text1 == text2,
            (Condition::Not(cond1), Condition::Not(cond2)) => cond1 == cond2,
            (Condition::And(left1, right1), Condition::And(left2, right2)) => {
                left1 == left2 && right1 == right2
            }
            (Condition::Or(left1, right1), Condition::Or(left2, right2)) => {
                left1 == left2 && right1 == right2
            }
            (Condition::AlwaysTrue, Condition::AlwaysTrue) => true,
            _ => false,
        }
    }
}

// Filter structs

#[derive(Clone, Debug)]
pub struct FilterCondition {
    pub expression: String,
    pub condition: Condition,
}

impl FilterCondition {
    pub fn new<S: Into<String>>(expression: S) -> Result<Self, String> {
        let expression = expression.into();
        let condition = parse_filter_expression(&expression)?;
        Ok(FilterCondition {
            expression,
            condition,
        })
    }
}

// Implement PartialEq for FilterCondition to enable comparison
impl PartialEq for FilterCondition {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression && self.condition == other.condition
    }
}

#[derive(Clone, Debug)]
pub struct Filter {
    pub id: Uuid,
    pub name: String,
    pub filter_condition: FilterCondition,
}

impl Filter {
    pub fn new<S: Into<String>>(name: S, filter_condition: FilterCondition) -> Self {
        Filter {
            id: Uuid::now_v7(),
            name: name.into(),
            filter_condition,
        }
    }
}

impl PartialEq for Filter {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.filter_condition == other.filter_condition
    }
}

// Parsing functions

pub fn parse_filter_expression(input: &str) -> Result<Condition, String> {
    if input.trim().is_empty() {
        return Ok(Condition::AlwaysTrue);
    }
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
                    preceded(multispace0, parse_or_operator),
                    preceded(multispace0, term),
                )),
            ),
            |(first, rest)| {
                rest.into_iter().fold(first, |acc, (_, term)| {
                    Condition::Or(acc.into(), term.into())
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
                    preceded(multispace0, parse_and_operator),
                    preceded(multispace0, factor),
                )),
            ),
            |(first, rest)| {
                rest.into_iter().fold(first, |acc, (_, factor)| {
                    Condition::And(acc.into(), factor.into())
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
                preceded(pair(parse_not_operator, multispace0), factor),
                |cond| Condition::Not(cond.into()),
            ),
            operand,
        )),
    )(input)
}

fn operand(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "operand",
        alt((
            map(
                delimited(
                    map(parse_parenthesis, |_| ()),
                    expression,
                    map(parse_parenthesis, |_| ()),
                ),
                |cond| cond,
            ),
            map(parse_completed, |_| {
                Condition::Completion(Completion::new(true))
            }),
            map(parse_incomplete, |_| {
                Condition::Completion(Completion::new(false))
            }),
            tag_condition,
            context_condition,
            text_condition,
        )),
    )(input)
}

fn tag_condition(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "tag condition",
        map(parse_tag, |token| {
            if let Token::Tag(tag_name) = token {
                Condition::Tag(Tag::new(tag_name))
            } else {
                unreachable!()
            }
        }),
    )(input)
}

fn context_condition(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "context condition",
        map(parse_context, |token| {
            if let Token::Context(context_name) = token {
                Condition::Context(Context::new(context_name))
            } else {
                unreachable!()
            }
        }),
    )(input)
}

fn text_condition(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    context(
        "text condition",
        map(parse_quoted_text, |token| {
            if let Token::QuotedText(text) = token {
                Condition::Text(Text::new(text.trim_matches('"')))
            } else {
                unreachable!()
            }
        }),
    )(input)
}

fn identifier(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    recognize(many1(alt((alphanumeric1, tag("."), tag("_"), tag("-")))))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::task::Task, utils::PersistentIndexMap};
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_task() -> Task {
        Task {
            id: Uuid::now_v7().into(),
            description: "Complete the urgent report #work @office"
                .to_string()
                .into(),
            tags: vec!["work".to_string()].into_iter().collect(),
            contexts: vec!["office".to_string()].into_iter().collect(),
            completed: None.into(),
            subtasks: PersistentIndexMap::new(),
        }
    }

    fn create_completed_task() -> Task {
        Task {
            id: Uuid::now_v7().into(),
            description: "Buy groceries #errand @shopping".to_string().into(),
            tags: vec!["errand".to_string()].into_iter().collect(),
            contexts: vec!["shopping".to_string()].into_iter().collect(),
            completed: Some(Utc::now()).into(),
            subtasks: PersistentIndexMap::new(),
        }
    }

    #[test]
    fn test_tag_condition() {
        let task = create_test_task();
        let condition = Tag::new("work");
        assert!(condition.evaluate(&task));

        let wrong_tag_condition = Tag::new("personal");
        assert!(!wrong_tag_condition.evaluate(&task));
    }

    #[test]
    fn test_context_condition() {
        let task = create_test_task();
        let condition = Context::new("office");
        assert!(condition.evaluate(&task));

        let wrong_context_condition = Context::new("home");
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
        let condition = Text::new("urgent");
        assert!(condition.evaluate(&task));

        let wrong_text_condition = Text::new("birthday");
        assert!(!wrong_text_condition.evaluate(&task));
    }

    #[test]
    fn test_not_condition() {
        let task = create_test_task();
        let tag_condition = Tag::new("work");
        let not_condition = Condition::Not(Condition::Tag(tag_condition).into());

        assert!(!not_condition.evaluate(&task));
    }

    #[test]
    fn test_and_condition() {
        let task = create_test_task();
        let tag_condition = Tag::new("work");
        let context_condition = Context::new("office");

        let and_condition = Condition::And(
            Condition::Tag(tag_condition).into(),
            Condition::Context(context_condition).into(),
        );
        assert!(and_condition.evaluate(&task));

        let wrong_and_condition = Condition::And(
            Condition::Tag(Tag::new("personal")).into(),
            Condition::Context(Context::new("office")).into(),
        );
        assert!(!wrong_and_condition.evaluate(&task));
    }

    #[test]
    fn test_or_condition() {
        let task = create_test_task();
        let tag_condition = Tag::new("work");
        let wrong_context_condition = Context::new("home");

        let or_condition = Condition::Or(
            Condition::Tag(tag_condition).into(),
            Condition::Context(wrong_context_condition).into(),
        );
        assert!(or_condition.evaluate(&task));

        let wrong_or_condition = Condition::Or(
            Condition::Tag(Tag::new("personal")).into(),
            Condition::Context(Context::new("home")).into(),
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

    #[test]
    fn test_parse_empty_expression() {
        let task = create_test_task();
        let completed_task = create_completed_task();

        // Empty expression should return Condition::AlwaysTrue
        let expression = "";
        let filter_condition = parse_filter_expression(expression).unwrap();
        assert!(filter_condition.evaluate(&task));
        assert!(filter_condition.evaluate(&completed_task));
    }

    #[test]
    fn test_parse_expression_with_and_or() {
        let task = create_test_task();
        let completed_task = create_completed_task();

        // Parse complex expression: (not [x] and #work) or (@home and "report")
        let expression = "(not [x] and #work) or (@home and \"report\")";
        let filter_condition = parse_filter_expression(expression).unwrap();

        // Ensure it evaluates to true for the incomplete task (not completed and has #work)
        assert!(filter_condition.evaluate(&task));

        // Ensure it evaluates to false for the completed task
        assert!(!filter_condition.evaluate(&completed_task));
    }

    #[test]
    fn test_parse_expression_with_nested_and_or() {
        let task = create_test_task();
        let completed_task = create_completed_task();

        // Parse complex expression: (not [x] and #work) or (([x] or @shopping) and "groceries")
        let expression = "(not [x] and #work) or (([x] or @shopping) and \"groceries\")";
        let filter_condition = parse_filter_expression(expression).unwrap();

        // Should evaluate to false for the incomplete task
        assert!(filter_condition.evaluate(&task));

        // Should evaluate to true for the completed task
        assert!(filter_condition.evaluate(&completed_task));
    }

    #[test]
    fn test_parse_expression_with_complex_nesting() {
        let task = create_test_task();
        let completed_task = create_completed_task();

        // Parse expression: (not [x] and (#work or "urgent")) or ([x] and @shopping)
        let expression = "(not [x] and (#work or \"urgent\")) or ([x] and @shopping)";
        let filter_condition = parse_filter_expression(expression).unwrap();

        // Should evaluate to true for the incomplete task (has #work and not completed)
        assert!(filter_condition.evaluate(&task));

        // Should evaluate to true for the completed task (completed and @shopping)
        assert!(filter_condition.evaluate(&completed_task));
    }

    #[test]
    fn test_parse_expression_with_not_and_and() {
        let task = create_test_task();
        let completed_task = create_completed_task();

        // Parse expression: not (#personal or @home) and not [x]
        let expression = "not (#personal or @home) and not [x]";
        let filter_condition = parse_filter_expression(expression).unwrap();

        // Should evaluate to true for the incomplete task (doesn't have #personal, @home or completed)
        assert!(filter_condition.evaluate(&task));

        // Should evaluate to false for the completed task
        assert!(!filter_condition.evaluate(&completed_task));
    }

    #[test]
    fn test_parse_expression_with_deep_nesting() {
        let task = create_test_task();
        let completed_task = create_completed_task();

        // Parse complex deeply nested expression:
        // (not [x] and (#work or "urgent")) and (not #personal) or ([x] and (@shopping or "groceries"))
        let expression = "(not [x] and (#work or \"urgent\")) and (not #personal) or ([x] and (@shopping or \"groceries\"))";
        let filter_condition = parse_filter_expression(expression).unwrap();

        // Should evaluate to true for the incomplete task (not completed, has #work, and no #personal)
        assert!(filter_condition.evaluate(&task));

        // Should evaluate to true for the completed task (completed, @shopping, and groceries)
        assert!(filter_condition.evaluate(&completed_task));
    }

    #[test]
    fn test_filter_condition_creation() {
        let expression = "#work and @office";
        let filter_condition = FilterCondition::new(expression).unwrap();

        assert_eq!(filter_condition.expression, expression);
        assert!(matches!(filter_condition.condition, Condition::And(_, _)));
    }

    #[test]
    fn test_filter_creation() {
        let expression = "#work";
        let filter_condition = FilterCondition::new(expression).unwrap();
        let filter = Filter::new("Work Filter", filter_condition.clone());

        assert_eq!(filter.name, "Work Filter");
        assert_eq!(filter.filter_condition, filter_condition);
    }

    #[test]
    fn test_parse_filter_expression_invalid() {
        let expression = "invalid_expression";
        let result = parse_filter_expression(expression);

        // Since "invalid_expression" does not match any valid operand, it should return an error
        assert!(result.is_err());
    }
}
