use std::iter::Scan;
use std::ops::Add;
use std::{collections::VecDeque, rc::Rc};

use yew::{Component, ComponentLink, Html, Properties, html};

use crate::zone::{Line, LineFilter, Scene, TextLine, TextPart, Zone};
use crate::raw::Raw;

pub struct State {
    log: VecDeque<String>,
    scene: Vec<String>,
    line: u32,
}

impl State {
    pub fn new() -> Self {
        Self {
            log: Default::default(),
            scene: vec![String::from("default")],
            line: 0
        }
    }
}


pub enum Message {
    LinkClick,
    NextLine,
}

#[derive(Properties, Clone)]
pub struct DisplayProperties {
    pub zone: Rc<Zone>
}

pub struct Display {
    link: ComponentLink<Self>,
    zone: Rc<Zone>,
    state: State,
}

impl Display {
    fn build_logs(&self) -> Html {
        let mut rows = Vec::new();
        for (index, log) in self.state.log.iter().enumerate() {
            rows.push(html!{<Raw key={index} inner_html={log.clone()}/>});
        }
        html!{
            <div class="logarea">
                {rows}
                {self.build_control()}
            </div>
        }
    }
    fn build_control(&self) -> Html {
        let scene = self.current_scene();
        if scene.branch {
            html!{<div>{format!("{:?}", scene)}</div>}
        } else {
            let line = &scene.lines[self.state.line as usize];
            html!{<div class="block"><Raw inner_html={self.render_active(line)} /></div>}
        }
    }
    fn build_inventory(&self) -> Html {
        html!{<Raw inner_html={"things"} />}
    }

    fn current_scene(&self) -> &Scene {
        let root = self.zone.find_scene(&self.state.scene[0]);
        self.part_from(&self.state.scene[1..], root)
    }

    fn part_from<'a>(&'a self, path: &[String], from: &'a Scene) -> &'a Scene {
        if path.len() == 0 {
            return from;
        }
        return self.part_from(&path[1..], from.find_section(&path[0]));
    }

    fn render_active(&self, line: &Line) -> String {
        match line {
            Line::TextLine(line) => self.render_text_line(line),
            Line::CommandLine(_) => todo!("command line"),
        }
    }

    fn render_text_line(&self, line: &TextLine) -> String {
        if let Some(filter) = &line.filter {
            if !self.check_filter(filter) {
                return String::from("");
            }
        }

        let mut out = String::from("");
        for part in &line.parts {
            match part {
                TextPart::Link(_) => todo!(),
                TextPart::Text(text) => out += text,
            }
        }
        return out;
    }

    fn check_filter(&self, filter: &LineFilter) -> bool {
        todo!()
    }
}

impl Component for Display {
    type Message = Message;
    type Properties = DisplayProperties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            zone: props.zone,
            state: State::new()
        }
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            Message::LinkClick => todo!(),
            Message::NextLine => todo!(),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> yew::ShouldRender {
        false
    }

    fn view(&self) -> yew::Html {
        html!{
            <div class="columns fullheight">
                <div class="column is-1"></div>
                <div class="column fullheight">
                    <div style="height:60%">{self.build_logs()}</div>
                    <div>{self.build_inventory()}</div>
                </div>
            </div>
        }
    }

}