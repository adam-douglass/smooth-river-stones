use std::collections::HashMap;
use std::ops::Deref;
use std::{collections::VecDeque, rc::Rc};

use web_sys::{KeyboardEvent, MouseEvent};
use yew::format::Json;
use yew::services::keyboard::KeyListenerHandle;
use yew::services::storage::Area;
use yew::services::{ConsoleService, KeyboardService, StorageService};
use yew::{Component, ComponentLink, Html, Properties, html};

use serde::{Deserialize, Serialize};

use crate::zone::{Command, FilterOperation, Line, LineFilter, Scene, TextLine, TextLink, TextPart, Zone};
use crate::raw::Raw;

#[derive(PartialEq, Deserialize, Serialize)]
enum Status {
    Running,
    Finished,
    Reset
}

#[derive(Deserialize, Serialize)]
pub struct State {
    log: VecDeque<String>,
    scene: String,
    line: usize,
    inventory: HashMap<String, i32>,
    visits: HashMap<String, u32>,
    values: HashMap<String, i32>,
    status: Status,
}

impl State {
    pub fn new() -> Self {
        Self {
            log: Default::default(),
            scene: String::from("default"),
            line: 0,
            inventory: Default::default(),
            visits: Default::default(),
            values: Default::default(),
            status: Status::Running,
        }
    }
}


pub enum Message {
    LinkClick(TextLink),
    NextLine(MouseEvent),
    Reset(MouseEvent),
    KeyboardEvent(KeyboardEvent),
}

#[derive(Properties, Clone)]
pub struct DisplayProperties {
    pub zone: Rc<Zone>
}

pub struct Display {
    link: ComponentLink<Self>,
    zone: Rc<Zone>,
    state: State,
    storage: Option<StorageService>,
    _event_handle: KeyListenerHandle
}

type DoAdvanceLine = bool;

impl Display {
    fn build_logs(&self) -> Html {
        let mut rows = Vec::new();        
        for log in self.state.log.iter() {
            rows.push(html!{<div class="dialog-line"><Raw inner_html={log.clone()} /></div>});
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
            Line::CommandLine(_) => None,
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
            FilterOperation::ReadVariable(name) => *self.state.values.get(name).unwrap_or(&0)
        }
    }

    // fn next_button(&self, children: Html) -> Html {
    //     let scene = self.current_scene();
    //     if scene.branch {
    //         html! {
    //             <div class="next-area">
    //                 <button class="button" disabled={true} style="cursor: auto">
    //                     <span class="icon">
    //                         <ion-icon size="large" name="help-circle"></ion-icon>
    //                     </span>
    //                 </button>
    //             </div>
    //         }
    //     } else {
    //         html! {
    //             <div class="next-area">
    //                 <button class="button" onclick={self.link.callback(Message::NextLine)}>
    //                     <span class="icon pulsing">
    //                         <ion-icon size="large" name="caret-forward"></ion-icon>
    //                     </span>
    //                 </button>
    //             </div>
    //         }
    //     }
    // }

    fn publish_link(&mut self, text: &String) {
        self.state.log.push_back(self.render_inactive_link(text))
    }

    fn render_inactive_link(&self, text: &String) -> String {
        String::from("<span class=\"inline-disabled-button\">") + &text + "</span>"
    }

    fn render_inactive(&self, line: &Line) -> String {
        match line {
            Line::TextLine(textline) => {
                let mut buffer = String::from("");
                for part in textline.parts.iter() {    
                    match part {
                        TextPart::Link(l) => buffer += &self.render_inactive_link(&l.text),
                        TextPart::Text(t) => buffer += t,
                    }
                }
                buffer
            },
            Line::CommandLine(_) => String::from(""),
        }
    }

    fn publish_current(&mut self){
        let line = {
            let scene = self.current_scene();
            if scene.branch {
                return
            } 
            let line = &scene.lines[self.state.line];
            self.render_inactive(line)
        };
        if line.len() > 0 {
            self.state.log.push_back(line);
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
            Command::Next(link) => {
                self.follow_link(link);
                false
            },
            Command::End => {
                self.state.status = Status::Finished;
                false
            },
            Command::Reset => {
                self.state.status = Status::Reset;
                false
            },
            Command::Set(cmd) => {
                self.state.values.insert(cmd.name.clone(), cmd.value);
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

    fn state_icon(&self) -> Html {
        match self.state.status {
            Status::Running => html!{},
            Status::Finished => html!{
                <span class="final-icon">
                    <ion-icon name="help-circle"></ion-icon>
                </span>
            },
            Status::Reset => html!{
                <span class="final-icon clickable-region" onclick={self.link.callback(Message::Reset)}>
                    <ion-icon name="refresh-circle"></ion-icon>
                </span>
            },
        }
    }

    fn save(&mut self) {
        if let Some(ss) = &mut self.storage {
            ss.store(STATE_KEY, Json(&self.state));
        }
    }
}

static STATE_KEY: &str = "session";

impl Component for Display {
    type Message = Message;
    type Properties = DisplayProperties;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local);
        if let Err(error) = storage {
            ConsoleService::error("Could not load local storage.");
            ConsoleService::error(error);
        }

        // Load the elapsed time
        let saved_state = match &storage {
            Ok(ss) => {
                let Json(elapsed_raw) = ss.restore(STATE_KEY);
                elapsed_raw.unwrap_or_else(|_| State::new())
            },
            Err(_) => State::new(),
        };

        let event_listener = KeyboardService::register_key_press(&web_sys::window().unwrap(), (&link).callback(|e: KeyboardEvent| Message::KeyboardEvent(e)));        
        Self {
            link,
            zone: props.zone,
            state: saved_state,
            storage: storage.ok(),
            _event_handle: event_listener
        }        
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            Message::LinkClick(target) => {
                if self.state.status == Status::Running {
                    ConsoleService::info(&format!("Click link: {}", target.destination));
                    self.publish_link(&target.text);
                    self.follow_link(&target.destination);
                    self.save();
                    true
                } else {
                    false
                }
            },
            Message::NextLine(_) => {
                if self.state.status == Status::Running {
                    ConsoleService::info("Next line");
                    self.publish_current();
                    self.advance_line(true);
                    self.save();
                    true
                } else {
                    false
                }
            },
            Message::Reset(_) => {
                ConsoleService::info("Reset");
                self.state = State::new();
                self.save();
                true
            },
            Message::KeyboardEvent(event) => {
                if self.state.status == Status::Running {
                    if !self.current_scene().branch {
                        if event.key() == " " {
                            self.publish_current();
                            self.advance_line(true);
                            return true
                        }
                    }
                }
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> yew::ShouldRender {
        false
    }

    fn view(&self) -> yew::Html {

        let scene = self.current_scene();
        let (background_click, background_class) = if !scene.branch && self.state.status == Status::Running {
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
                    {self.state_icon()}
                    {self.build_inventory()}
                </footer>
            </div>
        }
    }

}