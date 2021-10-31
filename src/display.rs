use std::{collections::VecDeque, rc::Rc};

use yew::{Component, ComponentLink, Html, Properties, html};

use crate::zone::Zone;
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
    LinkClick
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
        html!{<Raw inner_html={"old"} />}
    }
    fn build_control(&self) -> Html {
        html!{<Raw inner_html={"<i>current</i>"} />}
    }
    fn build_inventory(&self) -> Html {
        html!{<Raw inner_html={"things"} />}
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
        }
    }

    fn change(&mut self, _props: Self::Properties) -> yew::ShouldRender {
        false
    }

    fn view(&self) -> yew::Html {
        html!{
            <div class="columns">
                <div class="column is-1"></div>
                <div class="column">
                    {self.build_logs()}
                    {self.build_control()}
                    {self.build_inventory()}
                </div>
            </div>
        }
    }

}