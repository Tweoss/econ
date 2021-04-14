#![recursion_limit = "2048"]
use wasm_bindgen::prelude::*;
use yew::prelude::*;

extern crate console_error_panic_hook;
use anyhow::Error;
use std::panic;

use http::{Request, Response};
use yew::html::ComponentLink;
use yew::services::fetch;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;
use yew::services::{IntervalService, interval::IntervalTask};

use serde_cbor::{from_slice, to_vec};

use stdweb::js;

mod structs;
use structs::{
    ViewerClientMsg, ViewerClientType, ViewerServerMsg, ViewerServerType,
    Participant, /* Offsets,*/
};

struct Model {
    link: ComponentLink<Self>,
    ws: Option<WebSocketTask>,
    fetch_task: Option<fetch::FetchTask>,
    participants: Vec<Participant>,
    turn: u64,
    game_id: String,
    is_open: bool,
    is_unsorted: bool,
    trending: u8,
    subsidies: u8,
    supply_shock: u8,
    interval_task: Option<IntervalTask>,
}

impl Participant {
    fn render(&self, id: String, quantity: f64, link: ComponentLink<Model>) -> Html {
        html! {
            <>
            </>
        }
    }
}

trait ParticipantCollection {
    fn render(&self) -> Html;
    fn animate(&self) -> Html;
}

impl ParticipantCollection for Vec<Participant> {
    fn render(&self) -> Html {
        html! {
            <>
            </>
        }
    }
    fn animate(&self) -> Html {
        html! {
            <>
            </>
        }
    }
}

enum Msg {
    Connect(Vec<String>),                       // connect to websocket server
    Disconnected,                               // disconnected from server
    Ignore,                                     // ignore this message
    Received(Result<ViewerServerMsg, Error>), // data received from server
    PrepWsConnect,
    IntervalNotif,
}

impl Model {
    fn add_participant(&mut self, mut new_participants: Vec<Participant>) {
        self.participants.append(&mut new_participants);
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            ws: None,
            fetch_task: None,
            participants: Vec::new(),
            game_id: String::new(),
            is_open: false,
            is_unsorted: false,
            turn: 0,
            trending: 0,
            subsidies: 0,
            supply_shock: 0,
            interval_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connect(v) => {
                ConsoleService::log("Connecting");
                let cbout = self.link.callback(|data: Result<Vec<u8>, anyhow::Error>| {
                    Msg::Received(Ok(from_slice::<ViewerServerMsg>(&data.unwrap()).unwrap()))
                });
                let cbnot = self.link.batch_callback(|input| {
                    ConsoleService::log(&format!("Notification: {:?}", input));
                    match input {
                        WebSocketStatus::Closed | WebSocketStatus::Error => {
                            std::vec![Msg::Disconnected]
                        }
                        _ => std::vec![Msg::Ignore],
                    }
                });
                if self.ws.is_none() {
                    let window = web_sys::window;
                    let host: String = window().unwrap().location().host().unwrap();
                    let protocol: String = window().unwrap().location().protocol().unwrap();
                    let url = match protocol.as_str() {
                        "http:" => {
                            format!("ws://{}/ws/{}/{}/{}", host, v[0], v[1], v[2])
                        }
                        "https:" => {
                            format!("wss://{}/ws/{}/{}/{}", host, v[0], v[1], v[2])
                        }
                        &_ => return false,
                    };
                    let task = match WebSocketService::connect_binary(&url, cbout, cbnot) {
                        Err(e) => {
                            ConsoleService::error(e);
                            None
                        }
                        Ok(f) => Some(f),
                    };
                    self.ws = task;
                }
                true
            }
            Msg::Disconnected => {
                self.ws = None;
                ConsoleService::log("Disconnected");
                true
            }
            Msg::Ignore => false,
            Msg::Received(Ok(s)) => {
                match s.msg_type {
                    ViewerServerType::Info(mut info) => {
                        ConsoleService::log(&format!("{:?}", info));
                        self.participants.append(&mut info.participants);
                        self.turn = info.turn;
                        self.game_id = info.game_id;
                        self.is_open = info.is_open;
                        self.trending = info.trending;
                        self.subsidies = info.subsidies;
                        self.supply_shock = info.supply_shock;
                        self.interval_task = Some(IntervalService::spawn(std::time::Duration::from_secs(1), self.link.callback(|_| Msg::IntervalNotif)));
                    }
                    ViewerServerType::Ping => {
                        if let Some(ref mut task) = self.ws {
                            task.send_binary(Ok(to_vec(&ViewerClientMsg {
                                msg_type: ViewerClientType::Pong,
                            })
                            .unwrap()));
                        }
                        return false;
                    }
                    ViewerServerType::ServerKicked => {
                        self.ws = None;
                        js! {
                            document.getElementById("kick-modal").click();
                        }
                    }
                    ViewerServerType::GameEnded => {
                        js! {
                            document.getElementById("end-modal").click();
                        }
                    }
                    ViewerServerType::GameToggledOpen => {
                        self.is_open = !self.is_open;
                    }
                    ViewerServerType::TurnAdvanced => {
                        self.turn += 1;
                    }
                    ViewerServerType::NewOffsets(offsets) => {
                        self.trending = offsets.trending;
                        self.subsidies = offsets.subsidies;
                        self.supply_shock = offsets.supply_shock;
                    }
                    ViewerServerType::NewScores(vector) => {
                        // vector.iter().for_each(|x: (String, f64)| self.participants.iter_mut().position(|&participant| participant.));
                        self.is_unsorted = true;
                    }
                    ViewerServerType::NewParticipant(participant) => {
                        self.participants.push(participant);
                        self.is_unsorted = true;
                    }
                }
                true
            }
            Msg::Received(Err(_)) => {
                ConsoleService::log("Error reading information from WebSocket");
                true
            }
            Msg::PrepWsConnect => {
                let post_request = Request::post("/wsprep").body(yew::format::Nothing).unwrap();
                let callback = self
                    .link
                    .callback(|response: Response<Result<String, Error>>| {
                        if response.status().is_success() {
                            ConsoleService::log("Sent Request and Received Response with code: ");
                            ConsoleService::log(response.status().as_str());
                            let mut cookie_values: Vec<String> = Vec::new();
                            for cookie_value in response.body().as_ref().unwrap().split('\n') {
                                cookie_values.push(cookie_value.to_owned());
                            }
                            Msg::Connect(cookie_values)
                        } else {
                            ConsoleService::log("Failed to Send Websocket Open Request");
                            Msg::Ignore
                        }
                    });
                let fetch_task = fetch::FetchService::fetch(post_request, callback).unwrap();
                self.fetch_task = Some(fetch_task);
                false
            }
            Msg::IntervalNotif => {
                if self.ws.is_some() && self.is_unsorted {
                    self.participants.animate();
                    true
                }
                else {
                    false
                }
            }
        }
    }
    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }
    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link.send_message(Msg::PrepWsConnect);
        }
    }

    fn view(&self) -> Html {

        html! {
            <>

            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    App::<Model>::new().mount_to_body();
}
