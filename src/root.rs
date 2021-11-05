use std::rc::Rc;

use yew::services::ConsoleService;
use yew::{Component, ComponentLink, Html, ShouldRender, html};
use yew::services::fetch::{FetchTask, FetchService, Request, Response};
use yew::format::{Nothing, Text};

use crate::display::Display;
use crate::zone::{build_world, Zone};

pub enum Message {
    ZoneLoad(String),
    ZoneLoadError(anyhow::Error)
}

pub struct Root {
    _link: ComponentLink<Self>,
    _zone_fetch: Option<FetchTask>,
    load_error: Option<anyhow::Error>,
    zone: Option<Rc<Zone>>,
}

impl Component for Root {
    type Message = Message;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let doc = yew::utils::document();
        let host = match doc.url() {
            Ok(h) => h,
            Err(err) => {
                ConsoleService::error(&format!("Issue finding own host: {:?}", err));
                "http://localhost:8080".to_string()
            },
        };
        let url = host + "/static/index.zone";
        ConsoleService::info(&format!("Loading zone at: {}", url));
        
        let request = Request::get(url)
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
            _link: link,
            _zone_fetch: Some(task),
            zone: None,
            load_error: None,
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
                    <Display zone={zone} />
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