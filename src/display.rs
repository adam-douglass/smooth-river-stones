use std::collections::HashMap;
use std::ops::Deref;
use std::{collections::VecDeque, rc::Rc};

use web_sys::MouseEvent;
use yew::services::ConsoleService;
use yew::{Component, ComponentLink, Html, Properties, html};

use crate::zone::{Command, FilterOperation, Line, LineFilter, Scene, TextLine, TextLink, TextPart, Zone};
use crate::raw::Raw;

pub struct State {
    log: VecDeque<Html>,
    scene: String,
    line: usize,
    inventory: HashMap<String, i32>,
    visits: HashMap<String, u32>,
}

impl State {
    pub fn new() -> Self {
        Self {
            log: Default::default(),
            scene: String::from("default"),
            line: 0,
            inventory: Default::default(),
            visits: Default::default(),
        }
    }
}


pub enum Message {
    LinkClick(TextLink),
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
        for (_index, log) in self.state.log.iter().enumerate() {
            rows.push(log.clone());
        }
        html!{
            <div class="logarea">
                <div class="padding"></div>
                {rows}
                {self.build_control()}
            </div>
        }
    }
    fn build_control(&self) -> Html {
        let scene = self.current_scene();
        if scene.branch {
            let mut lines = Vec::new();
            for ll in &scene.lines {
                if let Some(view) = self.render_active(ll) {
                    lines.push(html!{<div class="dialog-line added-text">{view}</div>})
                }
            }
            html!{<>{lines}</>}
        } else {
            let line = &scene.lines[self.state.line as usize];
            if let Some(view) = self.render_active(line) {
                html!{<div class="dialog-line added-text">{view}</div>}
            } else {
                html!{<div class="dialog-line added-text"></div>}
            }
        }
    }
    fn build_inventory(&self) -> Html {
        let tags: Vec<Html> = self.state.inventory.iter().map(|(name, count)| html!{
            <span>{count}{" x "}<span class="tag">{name.clone()}</span></span>
        }).collect();
        html!{<div>
            {tags}
        </div>}
    }

    fn current_scene(&self) -> &Scene {
        self.zone.find_scene(&self.state.scene)
    }

    fn render_active(&self, line: &Line) -> Option<Html> {
        match line {
            Line::TextLine(line) => self.render_text_line(line),
            Line::CommandLine(_) => todo!("command line"),
        }
    }

    fn render_text_line(&self, line: &TextLine) -> Option<Html> {
        if let Some(filter) = &line.filter {
            if !self.check_filter(filter) {
                return None;
            }
        }

        let mut out = Vec::new();
        for part in &line.parts {
            out.push(match part {
                TextPart::Link(link) => {
                    let click = self.link.callback({
                        let dest = Rc::new(link.clone());
                        move |_| Message::LinkClick(dest.deref().clone())
                    });
                    html!{
                    <span class="inline-button" name={link.destination.clone()} onclick={click}>
                        <Raw inner_html={link.text.clone()}/>
                    </span>
                }},
                TextPart::Text(text) => html!{<Raw inner_html={text.clone()} />},
            });
        }
        return Some(html!{<>{out}</>});
    }

    fn check_filter(&self, filter: &LineFilter) -> bool {
        self.eval_filter(&filter.operation) != 0
    }

    fn eval_filter(&self, op: &FilterOperation) -> i32 {
        match op {
            FilterOperation::OperatorCall(call) => {
                match call.operator {
                    crate::zone::Ops::Add => self.eval_filter(&call.left) + self.eval_filter(&call.right),
                    crate::zone::Ops::Sub => self.eval_filter(&call.left) - self.eval_filter(&call.right),
                    crate::zone::Ops::Mul => self.eval_filter(&call.left) * self.eval_filter(&call.right),
                    crate::zone::Ops::Div => self.eval_filter(&call.left) / self.eval_filter(&call.right),
                    crate::zone::Ops::Gt => (self.eval_filter(&call.left) > self.eval_filter(&call.right)) as i32,
                    crate::zone::Ops::Gte => (self.eval_filter(&call.left) >= self.eval_filter(&call.right)) as i32,
                    crate::zone::Ops::Lt => (self.eval_filter(&call.left) < self.eval_filter(&call.right)) as i32,
                    crate::zone::Ops::Lte => (self.eval_filter(&call.left) <= self.eval_filter(&call.right)) as i32,
                    crate::zone::Ops::Eq => (self.eval_filter(&call.left) == self.eval_filter(&call.right)) as i32,
                    crate::zone::Ops::Ne => (self.eval_filter(&call.left) != self.eval_filter(&call.right)) as i32,
                }
            },
            FilterOperation::IntLiteral(lit) => *lit,
            FilterOperation::CountVisits(visit) => *self.state.visits.get(visit).unwrap_or(&0) as i32,
            FilterOperation::CountItems(item) => *self.state.inventory.get(item).unwrap_or(&0),
        }
    }

    fn next_button(&self, children: Html) -> Html {
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

    fn publish_link(&mut self, text: &String) {
        self.state.log.push_back(html!{
            <div class="dialog-line">
                <span class="inline-disabled-button">
                    <Raw inner_html={text.clone()}/>
                </span>
            </div>
        })
    }

    fn publish_current(&mut self){
        let line = {
            let scene = self.current_scene();
            if scene.branch {
                return
            } 
            let line = &scene.lines[self.state.line];
            self.render_active(line)
        };
        if let Some(view) = line {
            self.state.log.push_back(html!{<div class="dialog-line">{view}</div>});
        }
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

    fn follow_link(&mut self, link: &String) {
        self.state.line = 0;
        self.state.scene = link.clone();
        self.state.visits.insert(self.state.scene.clone(), 1 + self.count_visits(&self.state.scene));
        self.advance_line(false);
    }

    fn advance_scene(&mut self){
        self.state.line = 0;
        self.state.scene = self.zone.next(&self.state.scene);        
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

    fn count_visits(&self, name: &String) -> u32 {
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
            Message::LinkClick(target) => {
                ConsoleService::info(&format!("Click link: {}", target.destination));
                self.publish_link(&target.text);
                self.follow_link(&target.destination);
                true
            },
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

        let scene = self.current_scene();
        let (background_click, background_class) = if !scene.branch {
            (Some(self.link.callback(Message::NextLine)), "main-column clickable-region")
        } else {
            (None, "main-column")
        };

        html!{
            <div class={background_class} onclick={background_click}>
                <div class="main-row">
                    <div class="left-gutter"></div>
                    {self.build_logs()}
                </div>
                <footer class="footer-row">
                    {self.build_inventory()}
                </footer>
            </div>
        }
    }

}