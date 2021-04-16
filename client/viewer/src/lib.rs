#![recursion_limit = "2048"]
use std::convert::TryFrom;
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
use yew::services::{interval::IntervalTask, IntervalService};

use serde_cbor::{from_slice, to_vec};

use stdweb::js;

mod structs;
use structs::{
    Participant, /* Offsets,*/
    ViewerClientMsg, ViewerClientType, ViewerServerMsg, ViewerServerType,
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
    fn render(&self, index: usize, height: i32) -> Html {
        let offset =
            (i32::try_from(self.next_index).unwrap() - i32::try_from(index).unwrap()) * height;
        let string = if self.is_consumer {
            "Consumer"
        } else {
            "Producer"
        };
        html! {
            <tr style={&format!("transform: translateY({}px);", offset)}>
                <td>{self.next_index + 1}</td>
                <td>{&self.name}</td>
                <td>{format!("{:.2}", self.score)}</td>
                <td>{string}</td>
            </tr>
        }
    }
}

trait ParticipantCollection {
    fn render(&self) -> Html;
    fn sort(&mut self);
}

impl ParticipantCollection for Vec<Participant> {
    fn render(&self) -> Html {
        let document = web_sys::window().unwrap().document().unwrap();
        if let Ok(Some(first_row)) = document.query_selector("thead>tr") {
            let height = first_row.scroll_height();
            html! {
                <>
                    {for self.iter().enumerate().map(|(i, p)| p.render(i, height))}
                </>
            }
        } else {
            html! {
                <>
                </>
            }
        }
    }
    fn sort(&mut self) {
        let mut temp_vec: Vec<(f64, usize)> = self
            .iter()
            .enumerate()
            .map(|(index, participant)| (participant.score, index))
            .collect();
        temp_vec.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        for (new_index, elem) in temp_vec.iter().enumerate() {
            self[elem.1].next_index = new_index;
        }
    }
}

enum Msg {
    Connect(Vec<String>),                     // connect to websocket server
    Disconnected,                             // disconnected from server
    Ignore,                                   // ignore this message
    Received(Result<ViewerServerMsg, Error>), // data received from server
    PrepWsConnect,
    IntervalNotif,
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
                        self.participants.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
                        self.participants.sort(); // * sets all the indices to the correct value
                        self.turn = info.turn;
                        self.game_id = info.game_id;
                        self.is_open = info.is_open;
                        self.trending = info.trending;
                        self.subsidies = info.subsidies;
                        self.supply_shock = info.supply_shock;
                        // self.is_unsorted = true;
                        self.interval_task = Some(IntervalService::spawn(
                            std::time::Duration::from_secs(1),
                            self.link.callback(|_| Msg::IntervalNotif),
                        ));
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
                    ViewerServerType::GameOpened => {
                        self.is_open = true;
                    }
                    ViewerServerType::GameClosed => {
                        self.is_open = false;
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
                        vector.iter().for_each(|x: &(String, f64)| {
                            if let Some(p) = self
                                .participants
                                .iter_mut()
                                .find(|participant| participant.name == x.0)
                            {
                                p.score = x.1
                            }
                        });
                        self.is_unsorted = true;
                    }
                    ViewerServerType::NewParticipant(participant) => {
                        self.participants.push(participant);
                        self.is_unsorted = true;
                    }
                    ViewerServerType::KickedParticipant(participant) => {
                        self.participants.retain(|x| x.name != participant);
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
                    self.participants.sort();
                    self.is_unsorted = false;
                    true
                } else {
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
        let open = if self.is_open { "OPEN" } else { "CLOSED" };
        html! {
            <>
                <div class="container text-center flex-column" style="min-height: 100vh;width: 100vw;">
                    <h1>{"Viewer"}</h1>
                    <div class="row">
                        <div class="col-md-2">
                            <h2>{"Info"}</h2>
                            <p>{"Game ID: "}<strong>{&self.game_id}</strong></p>
                            <p>{"Game is "}<strong>{open}</strong></p>
                            <p>{"Turn: "}<strong>{&format!("{}", self.turn)}</strong></p>
                        </div>
                        <div class="col-md-8 d-flex flex-column">
                            <h2>{"Scoreboard"}</h2>
                            <div class="border rounded" id="list" style="max-height: 60vh;">
                                <div class="table-responsive" style="height: 100%;">
                                    <table class="table table-hover table-sm">
                                        <thead>
                                            <tr>
                                                <th>{"Rank"}</th>
                                                <th>{"Name"}</th>
                                                <th>{"Score"}</th>
                                                <th>{"Type"}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {self.participants.render()}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </div>
                        <div class="col-md-2">
                            <h2>{"Offset"}</h2>
                            <p>{"Trending: "}<strong>{&format!("{}", self.trending)}</strong></p>
                            <p>{"Subsidies: "}<strong>{&format!("{}", self.subsidies)}</strong></p>
                            <p>{"Supply Shock: "}<strong>{&format!("{}", self.supply_shock)}</strong></p>
                        </div>
                    </div>
                    <footer>
                        <p>{"Built by Francis Chua"}</p>
                    </footer>
                    <button class="btn btn-danger border rounded" id="kick-modal" type="button" data-toggle="modal" data-target="#kicked-modal" hidden=true></button>
                    <div class="modal fade" role="dialog" tabindex="-1" id="kicked-modal">
                        <div class="modal-dialog" role="document">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h4 class="modal-title">{"Kicked by Server"}</h4><button type="button" class="close" data-dismiss="modal" aria-label="Close"><span aria-hidden="true">{"×"}</span></button>
                                </div>
                                <div class="modal-footer">
                                    <a class="btn btn-info active" role="button" href="/director_login/index.html" id="test">{"Continue"}</a>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    App::<Model>::new().mount_to_body();
}
