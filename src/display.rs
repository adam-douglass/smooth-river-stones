use std::collections::HashMap;
use std::iter::Scan;
use std::ops::Add;
use std::{collections::VecDeque, rc::Rc};

use web_sys::MouseEvent;
use yew::services::ConsoleService;
use yew::{Component, ComponentLink, Html, Properties, html};

use crate::zone::{Command, Line, LineFilter, Scene, TextLine, TextPart, Zone};
use crate::raw::Raw;

pub struct State {
    log: VecDeque<String>,
    scene: Vec<String>,
    line: usize,
    inventory: HashMap<String, i32>,
    visits: HashMap<Vec<String>, u32>,
}

impl State {
    pub fn new() -> Self {
        Self {
            log: Default::default(),
            scene: vec![String::from("default")],
            line: 0,
            inventory: Default::default(),
            visits: Default::default(),
        }
    }
}


pub enum Message {
    LinkClick,
    NextLine(MouseEvent),
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

type DoAdvanceLine = bool;

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
        html!{<div></div>}
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

    fn next_button(&self) -> Html {
        let scene = self.current_scene();
        if scene.branch {
            html! {
                <div class="next-area">
                    <button class="button" disabled={true} style="cursor: auto">
                        <span class="icon">
                            <ion-icon size="large" name="help-circle"></ion-icon>
                        </span>
                    </button>
                </div>
            }
        } else {
            html! {
                <div class="next-area">
                    <button class="button" onclick={self.link.callback(Message::NextLine)}>
                        <span class="icon pulsing">
                            <ion-icon size="large" name="caret-forward"></ion-icon>
                        </span>
                    </button>
                </div>
            }
        }
    }

    fn publish_current(&mut self){
        self.state.log.push_back({
            let scene = self.current_scene();
            if scene.branch {
                return
            } 
            let line = &scene.lines[self.state.line];
            self.render_active(line)
        });
    }

    fn advance_line(&mut self, inc: bool) {
        if inc {
            self.state.line += 1;
        }
        let scene = { (*self.current_scene()).clone() };
        if scene.branch {
            return;
        }

        // Reached the end
        if scene.lines.len() <= self.state.line {
            self.advance_scene();
        }

        match &scene.lines[self.state.line] {
            Line::TextLine(line) => {
                // Skip lines that are filtered
                if let Some(filter) = &line.filter {
                    if !self.check_filter(filter) {
                        self.advance_line(true);
                    }
                }
            },
            Line::CommandLine(command) => {
                // Execute command line
                if self.execute_command(command) {
                    self.advance_line(true);
                }
            },
        }
    }

    fn advance_scene(&mut self){
        self.state.line = 0;
        let old_name = self.state.scene.pop().unwrap_or(String::from("default"));
        if self.state.scene.len() == 0 {
            self.state.scene.push(self.zone.next(&old_name));
        } else {
            self.state.scene.push(self.zone.next_in(&self.state.scene, &old_name));
        }
        self.state.visits.insert(self.state.scene.clone(), 1 + self.count_visits(&self.state.scene));
        self.advance_line(false);
    }

    fn execute_command(&mut self, command: &Command) -> DoAdvanceLine {
        match command {
            Command::Item(items) => {
                for (key, value) in items.change.iter() {
                    self.state.inventory.insert(key.clone(), self.count_item(key) + value);
                }
                true
            },
        }
    }

    fn count_item(&self, name: &String) -> i32 {
        match self.state.inventory.get(name) {
            Some(value) => *value,
            None => 0,
        }
    }

    fn count_visits(&self, name: &Vec<String>) -> u32 {
        match self.state.visits.get(name) {
            Some(value) => *value,
            None => 0,
        }
    }
}

impl Component for Display {
    type Message = Message;
    type Properties = DisplayProperties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            zone: props.zone,
            state: State::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            Message::LinkClick => todo!(),
            Message::NextLine(_) => {
                ConsoleService::info("Next line");
                self.publish_current();
                self.advance_line(true);
                true
            },
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
                    <div class="row">
                        {self.next_button()}
                        {self.build_inventory()}
                    </div>
                </div>
            </div>
        }
    }

}