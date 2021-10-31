use std::collections::{HashMap, VecDeque};

use yew::{Component, ComponentLink, Html, ShouldRender, html};
use yew::services::fetch::{FetchTask, FetchService, Request, Response};
use yew::format::{Nothing, Text};

use crate::nom_zone::{build_world, Zone};

pub enum Message {
    ZoneLoad(String),
    ZoneLoadError(anyhow::Error)
}

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

pub struct Root {
    link: ComponentLink<Self>,
    zone_fetch: Option<FetchTask>,
    zone: Option<Zone>,
    load_error: Option<anyhow::Error>,
    state: State,
}

impl Component for Root {
    type Message = Message;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {

        let request = Request::get("http://localhost:8080/static/index.zone")
            .body(Nothing)
            .expect("Could not build that request.");

        let callback = link.callback(|response: Response<Text>| {
            match response.into_body() {
                Ok(msg) => Message::ZoneLoad(msg),
                Err(err) => Message::ZoneLoadError(err),
            }
        });          
 
        let task = FetchService::fetch(request, callback).expect("failed to start request");                // 4. store the task so it isn't canceled immediately                self.fetch_task = Some(task);

        Self {
            link,
            zone_fetch: Some(task),
            zone: None,
            load_error: None,
            state: State::new()
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::ZoneLoad(load) => {
                self.zone = build_world(load);
                true    
            },
            Message::ZoneLoadError(err) => {
                self.load_error = Some(err);
                true
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if let Some(error) = &self.load_error {
            return html! {
                <div class="content">{error}</div>
            }
        }

        match &self.zone {
            Some(zone) => {
                html! {
                    <div class="content">{"Loaded"}</div>
                }
            },
            None => {
                html! {
                    <div class="content">{"Loading"}</div>
                }        
            },
        }
    }
}