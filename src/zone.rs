// use nom::branch::alt;
// use nom::bytes::complete::take_while1;
// use nom::character::complete::line_ending;
// use nom::complete::tag;
// // use nom::error::{ErrorKind, make_error};
// use nom::multi::{many0, many_till};
// use nom::sequence::tuple;
// use nom::{IResult, Err};
// use nom::combinator::eof;

// use yew::services::ConsoleService;



// pub enum Line {
//     TextLine(String),
//     CommandLine(String)
// }

// pub struct Scene {
//     label: String,
//     next: String,
//     branch: bool,
//     lines: Vec<Line>,
//     sections: Vec<Scene>
// }

// enum  Entry {
//     Line(Line),
//     Scene(Scene)
// }

// pub struct Zone {
//     scenes: Vec<Scene>
// }


// pub fn build_world(data: String) -> Option<Zone> {
//     ConsoleService::info("Parsing Zone");
//     ConsoleService::info(&data);
    
//     match parse_zone(&data) {
//         Ok(_) => todo!(),
//         Err(_) => todo!(),
//     }
// }

// // zone = ${ SOI ~ empty_line* ~ scene* ~ whitespace? ~ EOI }
// fn parse_zone(input: &str) -> IResult<&str, Zone> {
//     let (input, (values, _)) = many_till(parse_scene, eof)(input)?;
//     Ok((input, Zone{
//         scenes: values
//     }))
// }

// // scene = ${ dialog | branch }
// // dialog = ${ 
// //     label ~ line_end+ 
// //     ~ (dialog_multiple_lines | dialog_single_line)
// // }
// fn parse_scene(input: &str) -> IResult<&str, Scene> {
//     let (input, (label, _, _, lines)) = tuple((
//         label, line_end, many0(line_end), alt((dialog_multiple_lines, dialog_single_line))
//     ))(input)?;

//     Ok((input, Scene {}))
// }

// // label = ${ symbol ~ ":" }
// fn label(input: &str) -> IResult<&str, String> {
//     todo!()
// }

// // line_end = _{ whitespace? ~ endl }
// fn line_end(input: &str) -> IResult<&str, bool> {
//     let (in2, _) = tuple((tag(" "), many0(tag(" ")), line_ending))(input)?

//     Ok((in2, true))
// }

// // dialog_multiple_lines = ${ 
// //     PUSH(whitespace) ~ line 
// //     ~ (PEEK ~ line)*
// //     ~ POP ~ line
// // }
// fn dialog_multiple_lines(input: &str) -> IResult<&str, Vec<Entry>> {
//     todo!()
// }

// // dialog_single_line = ${ whitespace ~ line }
// fn dialog_single_line(input: &str) -> IResult<&str, Vec<Entry>> {
//     todo!()
// }


// // WHITESPACE = _{ " " }
// // COMMENT = _{ ("--" ~ (!"\n" ~ ANY)* ~ "\n") | (endl? ~ "<--" ~ (!"---" ~ ANY)* ~ "-->" ~ endl?) }

// // endl = ${ "\r"? ~ "\n" }
// // whitespace = _{ " "+ }

// // empty_line = _{ line_end | COMMENT }




// // branch = ${ 
// //     label ~ whitespace? ~ "??" ~ line_end+
// //     ~ PUSH(whitespace) ~ line 
// //     ~ (PEEK ~ line)*
// // }

// // symbol = ${ ASCII_ALPHANUMERIC ~ (ASCII_ALPHANUMERIC | "-")* } 



// // line = ${ (dialog | branch | command | text_line) ~ line_end+ }

// // command = ${ "*" ~ whitespace* ~ (item_command) }

// // item_command = ${"item" ~ (whitespace? ~ ("+" | "-") ~ whitespace? ~ symbol)+ }

// // text_line = ${ line_filter? ~ (text_fragment | link)+ }

// // text_fragment = { (ASCII_ALPHANUMERIC | "," | "<" | "/" | ">" | "(" | ")" | " " | ".")+ }

// // line_filter = { "(" ~ filter_expr ~ ")" }

// // link = !{ "[" ~ symbol ~ "|" ~ whitespace* ~ text_fragment ~ whitespace* ~ "]" }

// // filter_expr = !{ expr_equal } 

// // expr_equal = { expr_comp ~ (equal_operator ~ expr_comp)*}
// // equal_operator = {"=" | "!="}
// // expr_comp = { expr_sum ~ (comp_operator ~ expr_sum)*}
// // comp_operator = {">=" | ">" | "<" | "<="}
// // expr_sum = { expr_prod ~ (sum_operator ~ expr_prod )* }
// // sum_operator = {"+" | "-"}
// // expr_prod = { expr_atom ~ (prod_operator ~ expr_atom )* }
// // prod_operator = {"*" | "/"}
// // expr_atom = _{ ("(" ~ filter_expr ~ ")") | count_visits | int_literal }

// // count_visits = { "$" ~ symbol }

// // int_literal = { '1'..'9' ~ '0'..'9'* }

use pest::Parser;
use pest::iterators::Pairs;
// // use wasm_bindgen::prelude::Closure;
// // use web_sys::{Request, Response};
// // use wasm_bindgen::JsValue;
// // use wasm_bindgen::JsCast;
// // use wasm_bindgen_futures::JsFuture;
use yew::services::ConsoleService;
// // use std::boxed::{Box};
// // use std::error::Error;

#[derive(Parser)]
#[grammar = "zone.pest"]
pub struct ZoneParser;

enum Line {
    TextLine(String),
    CommandLine(String)
}

struct Scene {
    label: String,
    next: String,
    branch: bool,
    lines: Vec<Line>,
    sections: Vec<Scene>
}

pub struct Zone {
    scenes: Vec<Scene>
}

fn transform(root: Pairs<Rule>) -> Zone {
    ConsoleService::info(&format!("{:?}", root));
    Zone {
        scenes: Default::default()
    }
}

pub fn build_world(data: String) -> Option<Zone> {
    ConsoleService::info("Parsing Zone");
    ConsoleService::info(&data);
    

    match ZoneParser::parse(Rule::zone, &data) {
        Ok(root) => {
            ConsoleService::info("Parsed");
            return Some(transform(root));
        },
        Err(err) => {
            ConsoleService::error(&err.to_string());
            return None;
        },
    }
}