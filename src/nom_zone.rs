use std::collections::HashMap;
use std::hash::Hash;

use nom::branch::alt;
use nom::character::complete::{alphanumeric0, alphanumeric1, digit1, line_ending};
use nom::error::{VerboseError, convert_error};
// use nom::error::{ErrorKind, make_error};
use nom::multi::{many0, many1, many_till};
use nom::sequence::{pair, tuple};
use nom::bytes::complete::{is_a, tag};
use nom::{IResult, Err};
use nom::combinator::{eof, opt};

use yew::services::ConsoleService;

#[derive(Debug)]
pub struct ItemCommand {
    change: HashMap<String, i32>
}

#[derive(Debug)]
enum Command {
    Item(ItemCommand)
}

#[derive(Debug)]
enum Ops {
    Add,
    Sub,
    Mul,
    Div,
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Ne,
}

#[derive(Debug)]
struct OperatorCall {
    operator: Ops,
    left: FilterOperation,
    right: FilterOperation,
}

#[derive(Debug)]
enum FilterOperation {
    OperatorCall(Box<OperatorCall>),
    IntLiteral(i32),
    CountVisits(String),
    CountItems(String),
}

#[derive(Debug)]
struct LineFilter {
    operation: FilterOperation
}

#[derive(Debug)]
struct TextLink {
    destination: String,
    text: String
}

#[derive(Debug)]
enum TextPart {
    Link(TextLink),
    Text(String)
}

#[derive(Debug)]
pub struct TextLine {
    filter: Option<LineFilter>,
    parts: Vec<TextPart>
}

#[derive(Debug)]
pub enum Line {
    TextLine(TextLine),
    CommandLine(Command)
}

#[derive(Debug)]
pub struct Scene {
    label: String,
    next: String,
    branch: bool,
    lines: Vec<Line>,
    sections: Vec<Scene>
}

enum  Entry {
    Line(Line),
    Scene(Scene)
}

#[derive(Debug)]
pub struct Zone {
    scenes: Vec<Scene>
}


pub fn build_world(data: String) -> Option<Zone> {
    ConsoleService::info("Parsing Zone");
    ConsoleService::info(&data);
    
    match parse_zone(&data) {
        Ok((extra, zone)) => {
            if extra.len() > 0 {
                ConsoleService::error(&format!("Remaining {}", extra));
            }
            ConsoleService::info(&format!("{:#?}", zone));
            Some(zone)
        },
        Err(err) => {
            let mess = match err {
                Err::Incomplete(err) => "needed".to_string(),
                Err::Error(err) => convert_error::<&str>(&data, err),
                Err::Failure(err) => convert_error::<&str>(&data, err),
            };
            ConsoleService::error(&format!("{}", mess));
            None
        },
    }
}

type Result<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

// zone = ${ SOI ~ empty_line* ~ scene* ~ whitespace? ~ EOI }
fn parse_zone(input: &str) -> Result<Zone> {
    let (input, _) = many0(line_end)(input)?;
    let (input, (values, _)) = many_till(parse_scene, eof)(input)?;
    Ok((input, Zone{
        scenes: values
    }))
}

// scene = ${ dialog | branch }
// dialog = ${ 
//     label ~ "??" ~ line_end+ 
//     ~ (dialog_multiple_lines | dialog_single_line)
// }
fn parse_scene(input: &str) -> Result<Scene> {
    let (input, (label, _, query, _, entries)) = tuple((
        label, skip_ws, opt(tag("??")), many1(line_end), dialog_multiple_lines
    ))(input)?;
    let mut lines = Vec::new();
    let mut sections = Vec::new();
    for entry in entries {
        match entry {
            Entry::Line(line) => lines.push(line),
            Entry::Scene(scene) => sections.push(scene)
        }
    }

    Ok((input, Scene {
        label,
        next: Default::default(),
        branch: query.is_some(),
        lines,
        sections,
    }))
}

// label = ${ symbol ~ ":" }
fn label(input: &str) -> Result<String> {
    let (input, (text, _)) = pair(symbol, tag(":"))(input)?;
    Ok((input, text))
}

// line_end = _{ whitespace? ~ endl }
// empty_line = _{ line_end | COMMENT }
fn line_end(input: &str) -> Result<()> {   
    let (input, _) = pair(many0(tag(" ")), line_ending)(input)?;
    Ok((input, ()))
}

// dialog_multiple_lines = ${ 
//     PUSH(whitespace) ~ line 
//     ~ (PEEK ~ line)*
//     ~ POP ~ line
// }
fn dialog_multiple_lines(input: &str) -> Result<Vec<Entry>> {
    let (input, prefix) = is_a(" ")(input)?;
    let (input, first) = parse_entry(input)?;
    let (input, additional) = many0(pair(tag(prefix), parse_entry))(input)?;
    let mut shifted = vec![first];
    for (_, row) in additional {
        shifted.push(row);
    }
    return Ok((input, shifted));
}

// line = ${ (dialog | branch | command | text_line) ~ line_end+ }
fn parse_entry(input: &str) -> Result<Entry> {
    let (input, (entry, _)) = pair(alt((sub_block, command, text_line)), many0(line_end))(input)?;
    Ok((input, entry))
}

fn sub_block(input: &str) -> Result<Entry> {
    let (input, scene) = parse_scene(input)?;
    return Ok((input, Entry::Scene(scene)));
}

// symbol = ${ ASCII_ALPHANUMERIC ~ (ASCII_ALPHANUMERIC | "-")* } 
fn symbol(input: &str) -> Result<String> {
    let (input, (first, parts)) = pair(alphanumeric1, many0(alt((alphanumeric1, is_a("-")))))(input)?;
    Ok((input, first.to_string() + &parts.concat().to_string()))
}

// command = ${ "*" ~ whitespace* ~ (item_command) }
fn command(input: &str) -> Result<Entry> {
    let (input, (_, _, command)) = tuple((tag("*"), many0(tag(" ")), item_command))(input)?;
    let cmd = Command::Item(command);
    let line = Line::CommandLine(cmd);
    Ok((input, Entry::Line(line)))
}

// item_command = ${"item" ~ (whitespace? ~ ("+" | "-") ~ whitespace? ~ symbol)+ }
fn item_command(input: &str) -> Result<ItemCommand> {
    let (input, (_, parts)) = pair(tag("item"), many0(item_change))(input)?;
    let mut change = HashMap::new();
    for (name, value) in parts {
        match change.get_mut(&name) {
            Some(stored) => {
                *stored += value;
            },
            None => {
                change.insert(name, value);
            },
        }
    }
    Ok((input, ItemCommand{change}))
}

fn item_change(input: &str) -> Result<(String, i32)> {
    let (input, (_, change, _, name)) = tuple((many0(tag(" ")), alt((tag("+"), tag("-"))), many0(tag(" ")), symbol))(input)?;
    let value = if change == "+" {
        1
    } else {
        0
    };
    Ok((input, (name, value)))
}

// text_line = ${ line_filter? ~ (text_fragment | link)+ }
fn text_line(input: &str) -> Result<Entry> {
    let (input, (filter, body)) = pair(opt(line_filter), many1(text_part))(input)?;
    Ok((input, Entry::Line(Line::TextLine(TextLine{filter, parts: body}))))
}

fn text_part(input: &str) -> Result<TextPart> {
    alt((link, text_fragment))(input)
}

// text_fragment = { (ASCII_ALPHANUMERIC | "," | "<" | "/" | ">" | "(" | ")" | " " | ".")+ }
fn text_fragment(input: &str) -> Result<TextPart> {
    let (input, body) = raw_text_fragment(input)?;    
    Ok((input, TextPart::Text(body.to_string())))
}

fn raw_text_fragment(input: &str) -> Result<String> {
    let (input, body) = many1(alt((alphanumeric1, is_a(",'\"</>() ."))))(input)?;    
    Ok((input, body.concat().to_string()))
}

// link = !{ "[" ~ symbol ~ "|" ~ whitespace* ~ text_fragment ~ whitespace* ~ "]" }
fn link(input: &str) -> Result<TextPart> {
    let (input, (_, _, target, _, _, _, body, _, _)) = tuple((
        tag("["), skip_ws, symbol, skip_ws, tag("|"), skip_ws, raw_text_fragment, skip_ws, tag("]")
    ))(input)?;

    Ok((input, TextPart::Link(TextLink{destination: target, text: body})))
}

fn skip_ws(input: &str) -> Result<()> {
    let (input, _) = many0(tag(" "))(input)?;
    Ok((input, ()))
}

// line_filter = { "(" ~ filter_expr ~ ")" }
fn line_filter(input: &str) -> Result<LineFilter> {
    let (input, (_, _, filter, _, _)) = tuple((tag("("), skip_ws, filter_expr, skip_ws, tag(")")))(input)?;
    Ok((input, LineFilter{operation: filter}))
}


// filter_expr = !{ expr_equal } 
fn filter_expr(input: &str) -> Result<FilterOperation> {
    expr_equal(input)
}

// expr_equal = { expr_comp ~ (equal_operator ~ expr_comp)*}
fn expr_equal(input: &str) -> Result<FilterOperation> {
    let (input, (first, additional)) = pair(expr_comp, many0(pair(equal_operator, expr_comp)))(input)?;
    let mut out = first;
    for (op, expr) in additional {
        out = FilterOperation::OperatorCall(Box::new(OperatorCall{
            operator: op,
            left: out,
            right: expr
        }));
    }
    Ok((input, out))
}

// equal_operator = {"=" | "!="}
fn equal_operator(input: &str) -> Result<Ops> {
    let (input, (_, content, _)) = tuple((skip_ws, alt((tag("!="), tag("="))), skip_ws))(input)?;
    if content == "=" {
        Ok((input, Ops::Eq))
    } else {
        Ok((input, Ops::Ne))
    }
}

// expr_comp = { expr_sum ~ (comp_operator ~ expr_sum)*}
fn expr_comp(input: &str) -> Result<FilterOperation> {
    let (input, (first, additional)) = pair(expr_sum, many0(pair(comp_operator, expr_sum)))(input)?;
    let mut out = first;
    for (op, expr) in additional {
        out = FilterOperation::OperatorCall(Box::new(OperatorCall{
            operator: op,
            left: out,
            right: expr
        }));
    }
    Ok((input, out))
}

// comp_operator = {">=" | ">" | "<" | "<="}
fn comp_operator(input: &str) -> Result<Ops> {
    let (input, (_, content, _)) = tuple((skip_ws, alt((tag(">="), tag("<="), tag(">"), tag("<"))), skip_ws))(input)?;
    if content == ">=" {
        Ok((input, Ops::Gte))
    } else if content == "<=" {
        Ok((input, Ops::Lte))
    } else if content == "<" {
        Ok((input, Ops::Lt))
    } else {
        Ok((input, Ops::Gt))
    }
}


// expr_sum = { expr_prod ~ (sum_operator ~ expr_prod )* }
fn expr_sum(input: &str) -> Result<FilterOperation> {
    let (input, (first, additional)) = pair(expr_prod, many0(pair(sum_operator, expr_prod)))(input)?;
    let mut out = first;
    for (op, expr) in additional {
        out = FilterOperation::OperatorCall(Box::new(OperatorCall{
            operator: op,
            left: out,
            right: expr
        }));
    }
    Ok((input, out))
}

// sum_operator = {"+" | "-"}
fn sum_operator(input: &str) -> Result<Ops> {
    let (input, (_, content, _)) = tuple((skip_ws, alt((tag("+"), tag("-"))), skip_ws))(input)?;
    if content == "+" {
        Ok((input, Ops::Add))
    } else {
        Ok((input, Ops::Sub))
    }
}

// expr_prod = { expr_atom ~ (prod_operator ~ expr_atom )* }
fn expr_prod(input: &str) -> Result<FilterOperation> {
    let (input, (first, additional)) = pair(expr_atom, many0(pair(prod_operator, expr_atom)))(input)?;
    let mut out = first;
    for (op, expr) in additional {
        out = FilterOperation::OperatorCall(Box::new(OperatorCall{
            operator: op,
            left: out,
            right: expr
        }));
    }
    Ok((input, out))
}

// prod_operator = {"*" | "/"}
fn prod_operator(input: &str) -> Result<Ops> {
    let (input, (_, content, _)) = tuple((skip_ws, alt((tag("*"), tag("/"))), skip_ws))(input)?;
    if content == "*" {
        Ok((input, Ops::Mul))
    } else {
        Ok((input, Ops::Div))
    }
}

// expr_atom = _{ ("(" ~ filter_expr ~ ")") | count_visits | int_literal }
fn expr_atom(input: &str) -> Result<FilterOperation> {
    alt((sub_expr, count_visits, count_items, int_literal))(input)
}

fn sub_expr(input: &str) -> Result<FilterOperation> {
    let (input, (_, expr, _)) = tuple((tag("("), filter_expr, tag(")")))(input)?;
    Ok((input, expr))
}

fn count_items(input: &str) -> Result<FilterOperation> {
    let (input, (_, content)) = pair(tag("$"), symbol)(input)?;
    Ok((input, FilterOperation::CountItems(content)))
}

// count_visits = { "$" ~ symbol }
fn count_visits(input: &str) -> Result<FilterOperation> {
    let (input, (_, content)) = pair(tag("#"), symbol)(input)?;
    Ok((input, FilterOperation::CountVisits(content)))
}

// int_literal = { '1'..'9' ~ '0'..'9'* }
fn int_literal(input: &str) -> Result<FilterOperation> {
    let (input, content) = digit1(input)?;
    Ok((input, FilterOperation::IntLiteral(content.parse().unwrap())))
}
