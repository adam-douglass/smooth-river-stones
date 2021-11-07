use std::collections::HashMap;
use std::rc::Rc;

use yew::services::ConsoleService;
use yew::{Component, ComponentLink, Html, ShouldRender, html};
use yew::services::fetch::{FetchTask, FetchService, Request, Response};
use yew::format::{Nothing, Text};

use url::Url;
use anyhow::anyhow;

use crate::display::Display;
use crate::zone::{build_world, Zone};

pub enum Message {
    ZoneLoad(String),
    ZoneLoadError(anyhow::Error)
}

pub struct Root {
    _link: ComponentLink<Self>,
    _zone_fetch: Option<FetchTask>,
    session_key: String,
    load_error: Option<anyhow::Error>,
    zone: Option<Rc<Zone>>,
}

fn build_target_zone() -> Result<(Url, String), anyhow::Error> {
    let doc = yew::utils::document();
    let url = match doc.url() {
        Ok(url) => url,
        Err(err) => return Err(anyhow!(err.as_string().unwrap_or("Couldn't load document url".to_string())))
    };

    let mut url = match Url::parse(&url) {
        Ok(url) => url,
        Err(err) => return Err(anyhow!(err))
    };

    let query: HashMap<_, _> = url.query_pairs().collect();
    let path = match query.get("zone") {
        Some(value) => value.clone().into_owned(),
        None => "static/index.zone".to_string(),
    };
    let session = match query.get("session") {
        Some(value) => value.clone().into_owned(),
        None => "session".to_string(),
    };

    url.set_query(None);
    url.set_fragment(None);
    url = url.join(&path)?;

    return Ok((url, session));
}

impl Component for Root {
    type Message = Message;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let (url, session) = match build_target_zone() {
            Ok(url) => url,
            Err(err) => {
                return Self {
                    _link: link,
                    _zone_fetch: None,
                    zone: None,
                    session_key: String::from(""),
                    load_error: Some(err),
                }
            },
        };

        ConsoleService::info(&format!("Loading zone at: {}", url.to_string()));        
        ConsoleService::info(&format!("Using session: {}", session));        

        let request = Request::get(url.to_string())
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
            session_key: session,
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
                    <Display zone={zone} session_key={self.session_key.clone()} />
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