#![recursion_limit = "2048"]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventTarget;
use web_sys::HtmlParagraphElement;
// use wasm_bindgen::{JsCast};
use yew::prelude::*;

// use failure::Error;
use anyhow::Error;

use http::{Request, Response};
use yew::format::Json;
use yew::html::ComponentLink;
// use yew::prelude::*;
use yew::services::fetch;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

use serde_cbor::{from_slice, to_vec};

mod json;
use json::{DirectorClientMsg, DirectorClientType, DirectorServerMsg, DirectorServerType};

// use serde_json::json;
// use stdweb::js;

struct Model {
    link: ComponentLink<Self>,
    ws: Option<WebSocketTask>,
    server_data: String, // data received from the server
    text: String,        // text in our input box
    task: Option<fetch::FetchTask>,
    client_data: DirectorClientMsg,
    consumers: Vec<Participant>,
    producers: Vec<Participant>,
    directors: Vec<Participant>,
    viewers: Vec<Participant>,
    is_open: String,
    graph_data: Graphs,
}

struct Participant {
    id: String,
    state: PlayerState,
}

impl Participant {
    fn new(id: String) -> Participant {
        Participant {
            id,
            state: PlayerState::Disconnected,
        }
    }
    fn render(&self) -> Html {
        match self.state {
            PlayerState::Unresponsive => {
                html! {
                    <p class="kickable unresponsive">{self.id.clone()}</p>
                }
            }
            PlayerState::Connected => {
                html! {
                    <p class="kickable live">{self.id.clone()}</p>
                }
            }
            PlayerState::Disconnected => {
                html! {
                    <p class="kickable">{self.id.clone()}</p>
                }
            }
            PlayerState::Kicked => {
                html! {
                    <p class="kicked">{self.id.clone()}</p>
                }
            }
        }
    }
}

enum PlayerState {
    Unresponsive,
    Connected,
    Disconnected,
    Kicked,
}

// ! Make sure allowed graph values never goes below 0.
struct Graphs {
    consumer_x: f64,
    consumer_y: f64,
    producer_x: f64,
    producer_y: f64,
    trending: u8,
    supply_shock: u8,
    subsidies: u8,
}

impl Graphs {
    fn new() -> Graphs {
        Graphs {
            consumer_x: 51.25,
            consumer_y: 66.25,
            producer_x: 32.50,
            producer_y: 15.00,
            trending: 0,
            supply_shock: 0,
            subsidies: 0,
        }
    }
    fn data(&mut self, trending: u8, supply_shock: u8, subsidies: u8) {
        self.trending = trending;
        self.supply_shock = supply_shock;
        self.subsidies = subsidies;
    }
}

enum Msg {
    Connect(Vec<String>),                       // connect to websocket server
    Disconnected,                               // disconnected from server
    Ignore,                                     // ignore this message
    Received(Result<DirectorServerMsg, Error>), // data received from server
    SendReq,
    PrepWsConnect,
    FailedToConnect,
    EndGame,
    HandleClick(Option<EventTarget>),
    ToggleOpen,
}

// #[derive(Debug, Serialize, Deserialize)]
// struct Data {
//     choice: u64,
//     string: String,
// }

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            server_data: String::new(),
            ws: None,
            text: String::new(),
            task: None,
            client_data: DirectorClientMsg {
                msg_type: DirectorClientType::OpenGame,
                kick_target: None,
            },
            consumers: Vec::new(),
            producers: Vec::new(),
            directors: Vec::new(),
            viewers: Vec::new(),
            is_open: "Open".to_string(),
            graph_data: Graphs::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connect(v) => {
                ConsoleService::log("Connecting");
                let cbout = self.link.callback(|data: Result<Vec<u8>, anyhow::Error>| {
                    Msg::Received(Ok(from_slice::<DirectorServerMsg>(&data.unwrap()).unwrap()))
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
                    // ! SWITCH THIS REPL
                    let window = web_sys::window;
                    let host: String = window().unwrap().location().host().unwrap();
                    let url = format!("ws://{}/ws/{}/{}/{}",host, v[0], v[1], v[2]);
                    // let url = format!("ws://127.0.0.1:8080/ws/{}/{}/{}", v[0], v[1], v[2]);
                    // let url = format!("wss://web.valour.vision/ws/{}/{}/{}", v[0], v[1], v[2]);
                    // let task = match WebSocketService::connect("wss://web.valour.vision/ws", cbout, cbnot)
                    let task = match WebSocketService::connect_binary(&url, cbout, cbnot) {
                        Err(e) => {
                            ConsoleService::error(e);
                            None
                        }
                        Ok(f) => Some(f),
                    };
                    // let task = WebSocketService::connect("ws://127.0.0.1:8080/ws/", cbout, cbnot).unwrap();
                    // let task = self.wss.connect("ws://127.0.0.1:8080/ws/", cbout, cbnot.into());
                    // self.ws = Some(task);
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
                    DirectorServerType::Ping => {
                        if let Some(ref mut task) = self.ws {
                            ConsoleService::log("Sending Pong");
                            task.send_binary(Ok(to_vec(&DirectorClientMsg {
                                msg_type: DirectorClientType::Pong,
                                kick_target: None,
                            })
                            .unwrap()));
                        }
                        return false;
                    }
                    DirectorServerType::NewConsumer => {
                        self.consumers
                            .push(Participant::new(s.target.clone().unwrap()));
                    }
                    DirectorServerType::NewProducer => {
                        self.producers
                            .push(Participant::new(s.target.clone().unwrap()));
                    }
                    DirectorServerType::NewDirector => {
                        self.directors
                            .push(Participant::new(s.target.clone().unwrap()));
                    }
                    DirectorServerType::NewViewer => {
                        self.viewers
                            .push(Participant::new(s.target.clone().unwrap()));
                    }
                    DirectorServerType::ParticipantKicked => {
                        ConsoleService::log("Received Message to kick: ");
                        ConsoleService::log(&s.target.clone().unwrap());
                        //* remove any references to this id
                        self.producers
                            .retain(|x| &x.id != s.target.as_ref().unwrap());
                        self.consumers
                            .retain(|x| &x.id != s.target.as_ref().unwrap());
                        self.directors
                            .retain(|x| &x.id != s.target.as_ref().unwrap());
                        self.viewers.retain(|x| &x.id != s.target.as_ref().unwrap());
                    }
                    DirectorServerType::GameOpened => {
                        self.is_open = "Close".to_owned();
                    }
                    DirectorServerType::GameClosed => {
                        self.is_open = "Open".to_owned();
                    }
                    _ => {}
                }
                self.server_data.push_str(&format!("{:?}\n", s));
                true
            }
            Msg::Received(Err(s)) => {
                self.server_data.push_str(&format!(
                    "Error when reading data from server: {}\n",
                    &s.to_string()
                ));
                true
            }
            Msg::SendReq => match self.ws {
                Some(ref mut task) => {
                    task.send_binary(Ok(to_vec(&self.client_data).unwrap()));
                    true
                }
                None => false,
            },
            Msg::PrepWsConnect => {
                let post_request = Request::post("/wsprep")
                    // let post_request = Request::post("https://web.valour.vision/cookies")
                    .body(yew::format::Nothing)
                    .unwrap();
                let callback = self
                    .link
                    .callback(|response: Response<Result<String, Error>>| {
                        if response.status().is_success() {
                            // response.
                            ConsoleService::log("Sent Request and Received Response with code: ");
                            ConsoleService::log(response.status().as_str());
                            let mut cookie_values: Vec<String> = Vec::new();
                            for cookie_value in response.body().as_ref().unwrap().split('\n') {
                                cookie_values.push(cookie_value.to_owned());
                            }
                            Msg::Connect(cookie_values)
                        } else {
                            ConsoleService::log("Failed to Send Request");
                            Msg::FailedToConnect
                        }
                    });
                let task = fetch::FetchService::fetch(post_request, callback).unwrap();
                self.task = Some(task);
                false
            }
            Msg::FailedToConnect => {
                self.text.push_str("Failed to reach /wsprep");
                true
            }
            Msg::EndGame => match self.ws {
                Some(ref mut task) => {
                    task.send_binary(Ok(to_vec(&DirectorClientMsg {
                        msg_type: DirectorClientType::EndGame,
                        kick_target: None,
                    })
                    .unwrap()));
                    true
                }
                None => false,
            },
            Msg::HandleClick(possible_target) => {
                // if let Some(target) = possible_target {
                //     // if target.
                //     return true;
                // }
                match self.ws {
                    Some(ref mut task) => {
                        if let Some(target) = possible_target {
                            let element: HtmlParagraphElement =
                                target.dyn_ref::<HtmlParagraphElement>().unwrap().clone();
                            match element.class_name().as_ref() {
                                "kickable live" | "kickable unresponsive" | "kickable" => {
                                    // element.set_class_name("kicked");
                                    let iter = self.consumers.iter_mut().chain(self.producers.iter_mut()).chain(self.directors.iter_mut()).chain(self.viewers.iter_mut());
                                    for participant in iter {
                                        if participant.id == element.inner_html() {
                                            participant.state = PlayerState::Kicked;
                                        }
                                    }
                                    task.send_binary(Ok(to_vec(&DirectorClientMsg {
                                        msg_type: DirectorClientType::Kick,
                                        kick_target: Some(element.inner_html()),
                                    })
                                    .unwrap()));
                                    return true;
                                    // element.inner_text()
                                }
                                // "kickable unresponsive" => {
                                //     element.set_class_name("kicked");
                                //     task.send_binary(Ok(to_vec(&DirectorClientMsg {
                                //         msg_type: DirectorClientType::Kick,
                                //         kick_target: Some(element.inner_html()),
                                //     })
                                //     .unwrap()));
                                //     return true;
                                // }
                                // => {
                                //     element.set_class_name("kicked");
                                //     ConsoleService::log(&element.inner_html());
                                //     ConsoleService::log("Gonna kick: ");
                                //     task.send_binary(Ok(to_vec(&DirectorClientMsg {
                                //         msg_type: DirectorClientType::Kick,
                                //         kick_target: Some(element.inner_html()),
                                //     })
                                //     .unwrap()));
                                    // return true;
                                // }
                                _ => {}
                            }
                        }
                    }
                    None => return false,
                }
                // &web_sys::EventTarget) -> String {
                // if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
                // return input.value();
                // possible_target.is_prototype_of(&HtmlParagraphElement { obj:  });
                false
            }
            Msg::ToggleOpen => {
                if self.is_open == "Open" {
                    self.ws.as_mut().expect("No websocket open").send_binary(Ok(to_vec(&DirectorClientMsg {
                        msg_type: DirectorClientType::OpenGame,
                        kick_target: None,
                    })
                    .unwrap()));
                }
                else {
                    self.ws.as_mut().expect("No websocket open").send_binary(Ok(to_vec(&DirectorClientMsg {
                        msg_type: DirectorClientType::CloseGame,
                        kick_target: None,
                    })
                    .unwrap()));
                }
                false
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
        // let onbuttonconnect = self.link.callback(|_| Msg::PrepWsConnect);
        // let onbuttonsend = self.link.callback(|_| Msg::SendText);
        // let inputtext = self.link.callback(|e: InputData| Msg::TextInput(e.value));
        // let sendreq = self.link.callback(|_| Msg::SendReq);
        // let endgame = self.link.callback(|_| Msg::EndGame);
        let open_close = self.link.callback(|_| Msg::ToggleOpen);
        let handle_click = self
            .link
            .callback(|event: yew::MouseEvent| Msg::HandleClick(event.target()));

        html! {
            <>
                <div class="container text-center">
                <h1> {"Director Controls"}</h1>
                    <div class="row" style="margin-right: 0;margin-left: 0;">
                        <div class="col-md-4 text-center" style="padding: 0;min-height: 40vmin;">
                            <div class="row">
                                <div class="col" style="min-height: 40vmin;">
                                    <h2>{"Events"}</h2>
                                    <div class="btn-group-vertical btn-group-lg" role="group"><button class="btn btn-primary border rounded" type="button">{"Supply Shock"}</button><button class="btn btn-primary border rounded" type="button">{"Subsidies"}</button><button class="btn btn-primary border rounded" type="button">{"Trending"}</button></div>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col" style="min-height: 40vmin;">
                                    <h2>{"Control Flow"}</h2>
                                    <div class="btn-group-vertical btn-group-lg" role="group"><button class="btn btn-warning border rounded" type="button">{"Force Next Turn"}</button><button onclick=open_close class="btn btn-primary border rounded" type="button">{&self.is_open}</button><button class="btn btn-danger border rounded" type="button">{"End Game"}</button></div>
                                </div>
                            </div>
                        </div>
                        <div class="col-md-4 text-center" style="padding: 0;min-height: 40vmin;">
                            // <div class="d-flex flex-column" style="height: 100%;width: 100%;">
                            //     <h2>{"Graphs"}</h2>
                            //     <div class="d-xl-flex flex-fill justify-content-xl-center align-items-xl-center">
                            //     </div>
                            //     <div class="d-xl-flex flex-fill justify-content-xl-center align-items-xl-center" style="width: 100%;"><img/></div>
                            // </div>
                            <div class="d-flex flex-column" style="height: 100%;width: 100%;">
                                <h2>{"Graphs"}</h2>
                                <div class="d-xl-flex flex-fill justify-content-xl-center align-items-xl-center" style="width: 100%">
                                    <svg viewBox="-5 -5 100 100" preserveAspectRatio="xMidYMid meet" fill="white">
                                        <g transform="scale(1,-1) translate(0,-90)" style="cursor:cell">
                                            <rect width="105" height="105" x="-5" y="-5" fill-opacity="0%"></rect>
                                            
                                            <text x="10" y="-30" style="font: 10px Georgia; " transform="scale(1,-1)">{format!("{:.2},{:.2}",self.graph_data.consumer_x,self.graph_data.consumer_y)}</text>
    
                                            <path d={
                                                let temp: i16 = self.graph_data.trending.into();
                                                format!("M 0 {} C 40 {}, 70 {}, 80 {}", temp+80, temp+80, temp+70, temp)
                                            }  stroke="white" stroke-width="1" fill="transparent"/>
                                            
                                            <polygon points="0,95 -5,90 -1,90 -1,-1 90,-1 90,-5 95,0 90,5 90,1 1,1 1,90 5,90" fill="#1F6DDE" />
    
                                            <circle cx={format!("{:.2}",self.graph_data.consumer_x)} cy={format!("{:.2}",self.graph_data.consumer_y)} r="5" stroke="white" fill="#F34547" stroke-width="0.2" style="cursor:move"/>
                                        </g>
                                    </svg>
                                </div>
                                <div class="d-xl-flex flex-fill justify-content-xl-center align-items-xl-center" style="width: 100%;">
                                    <svg viewBox="-5 -5 100 100" preserveAspectRatio="xMidYMid meet" fill="white">
                                        <g transform="scale(1,-1) translate(0,-90)" style="cursor:cell">
                                            <rect width="105" height="105" x="-5" y="-5" fill-opacity="0%"></rect>
                                            <text x="10" y="-70" style="font: 10px Georgia; " transform="scale(1,-1)">{format!("{:.2},{:.2}",self.graph_data.producer_x,self.graph_data.producer_y)}</text>
                                            
                                            <path d={
                                                let net: i16 = i16::from(self.graph_data.subsidies) - i16::from(self.graph_data.supply_shock); 
                                                format!("M 0 {} C 10 {}, 50 {}, 80 {}", net+80, net-10, net-10, net+100)
                                            } stroke="white" stroke-width="1" fill="transparent"/>

                                            <polygon points="0,95 -5,90 -1,90 -1,-1 90,-1 90,-5 95,0 90,5 90,1 1,1 1,90 5,90" fill="#1F6DDE" />
    
                                            <circle cx={format!("{:.2}",self.graph_data.producer_x)} cy={format!("{:.2}",self.graph_data.producer_y)} r="5" stroke="white" fill="#F34547" stroke-width="0.2"/>
                                        </g>
                                            
                                    </svg>
                                </div>
                            </div>
                        </div>
                        <div onclick=handle_click class="col-md-4 text-center" style="padding: 0;min-height: 40vmin;">
                            <h2>{"State"}</h2>
                            <p>{"Game ID: 123456"}</p>
                            <p>{"Turn: 5"}</p>
                            <div id="participants" style="overflow-y: scroll;max-height: 50vh;">
                                <p class="lead" style="background: var(--dark);">{"Directors"}</p>
                                    {for self.directors.iter().map(|elem| elem.render())}
                                <p class="lead" style="background: var(--dark);">{"Viewers"}</p>
                                    {for self.viewers.iter().map(|elem| elem.render())}
                                <p class="lead" style="background: var(--dark);">{"Consumers"}</p>
                                    {for self.consumers.iter().map(|elem| elem.render())}
                                <p class="lead" style="background: var(--dark);">{"Producers"}</p>
                                    {for self.producers.iter().map(|elem| elem.render())}
                            </div>
                        </div>
                    </div>
                    <footer>
                        <p>{"Built by Francis Chua"}</p>
                    </footer>
            </div>
                // for elem in self.producers {
                //     elem.render();
                // }
                // for elem in self.directors {
                //     elem.render();
                // }
                // for elem in self.viewers {
                //     elem.render();
                // }
            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
