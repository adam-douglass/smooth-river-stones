use std::collections::HashMap;
use std::rc::Rc;

use nom::branch::alt;
use nom::character::complete::{alphanumeric1, digit1, line_ending};
use nom::error::{VerboseError, convert_error};
use nom::multi::{many0, many1, many_till};
use nom::sequence::{pair, tuple};
use nom::bytes::complete::{is_a, is_not, tag};
use nom::{IResult, Err};
use nom::combinator::{eof, opt};

use yew::services::ConsoleService;

fn parent(val: &String) -> String {
    match val.rfind('.') {
        Some(point) => {
            val[0..point].to_string()
        },
        None => String::from(""),
    }
}


#[derive(Debug, Clone)]
pub struct ItemCommand {
    pub change: HashMap<String, i32>
}

#[derive(Debug, Clone)]
pub enum Command {
    Item(ItemCommand)
}

#[derive(Debug, Clone)]
pub enum Ops {
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

#[derive(Debug, Clone)]
pub struct OperatorCall {
    pub operator: Ops,
    pub left: FilterOperation,
    pub right: FilterOperation,
}

#[derive(Debug, Clone)]
pub enum FilterOperation {
    OperatorCall(Box<OperatorCall>),
    IntLiteral(i32),
    CountVisits(String),
    CountItems(String),
}

#[derive(Debug, Clone)]
pub struct LineFilter {
    pub operation: FilterOperation
}

#[derive(Debug, Clone)]
pub struct TextLink {
    pub destination: String,
    pub text: String
}

#[derive(Debug, Clone)]
pub enum TextPart {
    Link(TextLink),
    Text(String)
}

#[derive(Debug, Clone)]
pub struct TextLine {
    pub filter: Option<LineFilter>,
    pub parts: Vec<TextPart>
}

#[derive(Debug, Clone)]
pub enum Line {
    TextLine(TextLine),
    CommandLine(Command)
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub label: String,
    pub branch: bool,
    pub lines: Vec<Line>,
}

impl Scene {
    fn _update_labels(&mut self, names: &Vec<String>){
        self.lines = self.lines.clone().into_iter().map(|mut line|{
            match &mut line {
                Line::TextLine(text) => {
                    if let Some(filter) = &mut text.filter {
                        self._update_filter_operation_labels(&names, &mut filter.operation);
                    }
                    for part in &mut text.parts {
                        match part {
                            TextPart::Link(link) => {
                                link.destination = self._fix_label(&names, &link.destination)
                            },
                            TextPart::Text(_) => {},
                        }
                    }
                },
                Line::CommandLine(command) => {
                    match command {
                        Command::Item(_) => {},
                    }
                },
            }
            line
        }).collect();

        // for line in self.lines.iter_mut() {
        //     match line {
        //         Line::TextLine(text) => {
        //             if let Some(filter) = &mut text.filter {
        //                 self._update_filter_operation_labels(&names, &mut filter.operation);
        //             }
        //             for part in &mut text.parts {
        //                 match part {
        //                     TextPart::Link(link) => {
        //                         link.destination = self._fix_label(&names, &link.destination)
        //                     },
        //                     TextPart::Text(_) => {},
        //                 }
        //             }
        //         },
        //         Line::CommandLine(command) => {
        //             match command {
        //                 Command::Item(_) => {},
        //             }
        //         },
        //     }
        // }
    }

    fn _update_filter_operation_labels(&mut self, names: &Vec<String>, op: &mut FilterOperation) {
        match op {
            FilterOperation::OperatorCall(call) => {
                self._update_filter_operation_labels(&names, &mut call.left);
                self._update_filter_operation_labels(&names, &mut call.right);
            },
            FilterOperation::IntLiteral(_) => {},
            FilterOperation::CountVisits(count) => {
                *count = self._fix_label(&names, &count);
            },
            FilterOperation::CountItems(_) => {},
        }
    }

    fn _fix_label(&self, names: &Vec<String>, old: &String) -> String {
        let mut prefix = self.label.clone() + ".1";
        while prefix.len() > 0 {            
            prefix = parent(&prefix);
            let mut alt = prefix.clone() + "." + old;
            alt = alt.strip_prefix(".").unwrap_or(&alt).to_string();
            if names.contains(&alt){
                return alt;
            }    
        }

        ConsoleService::error(&format!("Bad link '{}' in '{}'", old, self.label));
        return old.clone();
    }
}

#[derive(Debug)]
pub struct Zone {
    scenes: Vec<Scene>,
    lookup: HashMap<String, usize>,
}

impl Zone {
    fn new(scenes: Vec<Scene>) -> Self {
        let lookup = scenes.iter().enumerate().map(|(i, s)| (s.label.clone(), i)).collect();
        Self {
            scenes,
            lookup,
        }
    }

    fn scene_names(&self) -> Vec<String> {
        self.scenes.iter()
            .map(|s| s.label.clone())
            .collect()
    }

    fn correct(&mut self) {
        let names = self.scene_names();
        for sec in &mut self.scenes {
            sec._update_labels(&names);
        }
    }

    pub fn find_scene(&self, name: &String) -> &Scene {
        if let Some(&index) = self.lookup.get(name) {
            return &self.scenes[index];
        }
        ConsoleService::error(&format!("Bad scene name: {}", name));
        panic!{}
    }

    // pub fn link_to(&self, at: &Vec<String>, link: String) -> Vec<String> {
    //     todo!{}
    // }

    pub fn next(&self, last: &String) -> String {
        let here = self.find_scene(last);
        let up = parent(&here.label);
        if up != "" && self.find_scene(&up).branch {
            return up;
        }
        let start = self.lookup.get(last).unwrap_or(&0) + 1;
        for scene in self.scenes[start..].iter() {
            if parent(&scene.label) == up {
                return scene.label.clone();
            }
        }
        ConsoleService::error(&format!("Bad scene name: {}", last));
        panic!()
    }

    // pub fn next_in(&self, path: &Vec<String>, last: &String) -> String {
    //     todo!{}
    // }
}


pub fn build_world(data: String) -> Option<Rc<Zone>> {
    ConsoleService::info("Parsing Zone");
    ConsoleService::info(&data);
    
    match parse_zone(&data) {
        Ok((extra, mut zone)) => {
            if extra.len() > 0 {
                ConsoleService::error(&format!("Remaining {}", extra));
            }

            for name in zone.scene_names(){
                ConsoleService::info(&format!("{:}", name));                
            }

            zone.correct();            
            ConsoleService::info(&format!("{:#?}", zone));
            Some(Rc::new(zone))
        },
        Err(err) => {
            let mess = match err {
                Err::Incomplete(_) => "needed".to_string(),
                Err::Error(err) => convert_error::<&str>(&data, err),
                Err::Failure(err) => convert_error::<&str>(&data, err),
            };
            ConsoleService::error(&format!("{}", mess));
            None
        },
    }
}

type Result<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

enum  Entry {
    Line(Line),
    Scene(Vec<Scene>)
}

// zone = ${ SOI ~ empty_line* ~ scene* ~ whitespace? ~ EOI }
fn parse_zone(input: &str) -> Result<Zone> {
    let (input, _) = many0(line_end)(input)?;
    let (input, (values, _)) = many_till(parse_scene, eof)(input)?;

    Ok((input, Zone::new(values.concat())))
}

// scene = ${ dialog | branch }
// dialog = ${ 
//     label ~ "??" ~ line_end+ 
//     ~ (dialog_multiple_lines | dialog_single_line)
// }
fn parse_scene(input: &str) -> Result<Vec<Scene>> {
    let (input, (label, _, query, _, entries)) = tuple((
        label, skip_ws, opt(tag("??")), many1(line_end), dialog_multiple_lines
    ))(input)?;

    let mut lines = Vec::new();
    let mut sections = Vec::new();
    for entry in entries {
        match entry {
            Entry::Line(line) => lines.push(line),
            Entry::Scene(scenes) => {
                for mut scene in scenes {
                    scene.label = label.clone() + "." + &scene.label;
                    sections.push(scene)
                }
            }
        }
    }

    let mut out = vec![Scene {
        label,
        branch: query.is_some(),
        lines,
    }];
    out.append(&mut sections);

    Ok((input, out))
}

// label = ${ symbol ~ ":" }
fn label(input: &str) -> Result<String> {
    let (input, (text, _)) = pair(symbol, tag(":"))(input)?;
    Ok((input, text))
}

// line_end = _{ whitespace? ~ endl }
// empty_line = _{ line_end | COMMENT }
fn line_end(input: &str) -> Result<()> {   
    let (input, _) = tuple((many0(tag(" ")), opt(comment), line_ending))(input)?;
    Ok((input, ()))
}

fn comment(input: &str) -> Result<()> {   
    let (input, _) = tuple((tag("---"), is_not("\n\r")))(input)?;
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

fn count_visits(input: &str) -> Result<FilterOperation> {
    let (input, (_, content)) = pair(tag("#"), symbol)(input)?;
    Ok((input, FilterOperation::CountVisits(content)))
}

// int_literal = { '1'..'9' ~ '0'..'9'* }
fn int_literal(input: &str) -> Result<FilterOperation> {
    let (input, content) = digit1(input)?;
    Ok((input, FilterOperation::IntLiteral(content.parse().unwrap())))
}
