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

use crate::zone::{Command, FilterOperation, Item, Line, LineFilter, Scene, TextLine, TextLink, TextPart, Zone};
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
    items: HashMap<String, Item>,
    visits: HashMap<String, u32>,
    values: HashMap<String, i32>,
    status: Status,
}

impl State {
    pub fn new(init: &Vec<Command>) -> Self {
        Self {
            log: Default::default(),
            scene: String::from("default"),
            line: 0,
            inventory: Default::default(),
            items: init.iter().fold(Default::default(), |mut acc, val|{
                if let Command::SetItem(item) = val {
                    acc.insert(item.key.clone(), item.clone());
                }
                acc
            }),
            visits: Default::default(),
            values: init.iter().fold(Default::default(), |mut acc, val|{
                if let Command::Set(cmd) = val {
                    acc.insert(cmd.name.clone(), cmd.value);
                }
                acc
            }),
            status: Status::Running,
        }
    }
}


pub enum Message {
    LinkClick(MouseEvent, TextLink),
    NextLine(MouseEvent),
    Reset(MouseEvent),
    KeyboardEvent(KeyboardEvent),
}

#[derive(Properties, Clone)]
pub struct DisplayProperties {
    pub zone: Rc<Zone>,
    pub session_key: String,
}

pub struct Display {
    link: ComponentLink<Self>,
    zone: Rc<Zone>,
    state: State,
    storage: Option<StorageService>,
    session_key: String,
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
        let tags: Vec<Html> = self.state.inventory.iter().map(|(name, count)| {
            if let Some(item) = self.state.items.get(name) {
                if let Some(name) = &item.name {
                    let details = if let Some(detail) = &item.details {
                        if detail.len() > 0 {
                            html!{<div class="infoboxtext"><div class="infoboxinner">{detail.clone()}</div></div>}
                        } else {
                            html!{}
                        }
                    } else {
                        html!{}
                    };

                    let counter = if count > &1 {
                        html!{<span class="label">{count}{" x "}</span>}
                    } else {
                        html!{}
                    };

                    return html!{
                        <span class="item infobox">
                            {counter}
                            <span class="tag">{name.clone()}</span>
                            {details}
                        </span>
                    }
                }
            }
            html!{
                <span class="item"><span class="label">{count}{" x "}</span><span class="tag">
                    {name}
                </span></span>
            }
        }).collect();
        html!{<div class="item-box">
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
                        move |x| Message::LinkClick(x, dest.deref().clone())
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
                    crate::zone::Ops::And => 
                        if self.eval_filter(&call.left) != 0 {
                            self.eval_filter(&call.right)
                        } else {
                            0
                        },
                    crate::zone::Ops::Or => {
                        let left = self.eval_filter(&call.left);
                        if left != 0 {
                            left
                        } else {
                            self.eval_filter(&call.right)
                        }
                    },
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

    fn publish_link(&mut self, link: &TextLink) {
        let scene = self.current_scene().clone();
        if scene.branch {
            for line in scene.lines.iter() {
                if let Line::TextLine(text) = line {
                    if text.include_in_summary {
                        self.state.log.push_back(self.render_inactive(&line))
                    }
                }
            }
        }
        self.state.log.push_back(self.render_inactive_link(&link.text))
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
            Command::SetItem(item) => {
                match self.state.items.get_mut(&item.key) {
                    Some(val) => {
                        val.update(item);
                    },
                    None => {
                        self.state.items.insert(item.key.clone(), item.clone());
                    },
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
            ss.store(&self.session_key, Json(&self.state));
        }
    }
}

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
                let Json(elapsed_raw) = ss.restore(&props.session_key);
                elapsed_raw.unwrap_or_else(|_| State::new(&props.zone.initialize))
            },
            Err(_) => State::new(&props.zone.initialize),
        };

        let event_listener = KeyboardService::register_key_press(&web_sys::window().unwrap(), (&link).callback(|e: KeyboardEvent| Message::KeyboardEvent(e)));        
        Self {
            link,
            zone: props.zone,
            state: saved_state,
            storage: storage.ok(),
            session_key: props.session_key,
            _event_handle: event_listener
        }        
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            Message::LinkClick(event, target) => {
                event.stop_propagation();
                if self.state.status == Status::Running {
                    ConsoleService::info(&format!("Click link: {}", target.destination));
                    self.publish_link(&target);
                    self.follow_link(&target.destination);
                    self.save();
                    true
                } else {
                    false
                }
            },
            Message::NextLine(event) => {
                event.stop_propagation();
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
            Message::Reset(event) => {
                event.stop_propagation();
                ConsoleService::info("Reset");
                self.state = State::new(&self.zone.initialize);
                self.save();
                true
            },
            Message::KeyboardEvent(event) => {
                event.stop_propagation();
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