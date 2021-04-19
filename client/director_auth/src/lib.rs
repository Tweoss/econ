#![recursion_limit = "4096"]
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventTarget;
use web_sys::{HtmlParagraphElement, SvggElement};
// use wasm_bindgen::{JsCast};
use yew::prelude::*;

// use failure::Error;
use anyhow::Error;
extern crate console_error_panic_hook;
use std::panic;

use http::{Request, Response};
use yew::html::ComponentLink;
// use yew::prelude::*;
use yew::services::fetch;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

use serde_cbor::{from_slice, to_vec};

use stdweb::js;

mod structs;
use structs::{
    DirectorClientMsg, DirectorClientType, DirectorServerMsg, DirectorServerType, Offsets,
};

use structs::{Participant, PlayerState};

// use serde_json::json;
// use stdweb::js;

struct Model {
    link: ComponentLink<Self>,
    ws: Option<WebSocketTask>,
    // server_data: String, // data received from the server
    text: String, // text in our input box
    task: Option<fetch::FetchTask>,
    // client_data: DirectorClientMsg,
    consumers: HashMap<String, Participant>,
    producers: HashMap<String, Participant>,
    directors: HashMap<String, Participant>,
    viewers: HashMap<String, Participant>,
    is_open: String,
    graph_data: Graphs,
    turn: u64,
    game_id: String,
    personal_id: String,
    winners: Option<[Option<Vec<(String, String)>>; 3]>,
}

impl Participant {
    fn new(can_take_turn: bool, id: String) -> Participant {
        if can_take_turn {
            Participant {
                state: PlayerState::Disconnected,
                took_turn: Some(false),
                id,
            }
        } else {
            Participant {
                state: PlayerState::Disconnected,
                took_turn: None,
                id,
            }
        }
    }
    fn render(&self, name: String) -> Html {
        let icon = match self.took_turn {
            Some(true) => html! {<i class="fa fa-check"></i>},
            Some(false) => html! {<i class="fa fa-remove"></i>},
            None => html! {<></>},
        };
        match self.state {
            PlayerState::Unresponsive => {
                html! {
                    <p class="kickable unresponsive" id={name.clone()}>{format!("{:?}, {} ", name.clone(), self.id)} <i class="fa fa-signal"></i> {icon} </p>
                }
            }
            PlayerState::Connected => {
                html! {
                    <p class="kickable live" id={name.clone()}>{format!("{:?}, {} ", name.clone(), self.id)} <i class="fa fa-user"></i> {icon} </p>
                }
            }
            PlayerState::Disconnected => {
                html! {
                    <p class="kickable" id={name.clone()}>{format!("{:?}, {} ", name.clone(), self.id)}<i class="fa fa-user-o"></i> {icon} </p>
                }
            }
            PlayerState::Kicked => {
                html! {
                    <p class="kicked" id={name.clone()}>{format!("{:?}, {} ", name.clone(), self.id)}</p>
                }
            }
        }
    }
}

trait ParticipantCollection {
    fn render(&self) -> Html;
    fn update_status(&mut self, name: &str, status: PlayerState);
    fn took_turn(&mut self, name: &str);
}

impl ParticipantCollection for HashMap<String, Participant> {
    fn render(&self) -> Html {
        html! {
            <>
                {for self.keys().zip(self.values()).map(|tuple| tuple.1.render(tuple.0.to_string()))}

            </>
        }
    }
    fn update_status(&mut self, name: &str, status: PlayerState) {
        if let Some(participant) = self.get_mut(name) {
            participant.state = status;
        }
    }
    fn took_turn(&mut self, name: &str) {
        if let Some(participant) = self.get_mut(name) {
            participant.took_turn = Some(true);
        }
    }
}

trait WinningRenders {
    fn render_table(&self) -> Html;
    fn render_csv(&self) -> Html;
}

fn render_single(place: u8, name: &str, hash: &str) -> Html {
    let string = match place {
        0 => "first",
        1 => "second",
        2 => "third",
        _ => ""
    };
    html! {
        <>
            <br/>
            {format!("{},{},{}", name, string, hash)}
        </>
    }
}

impl WinningRenders for Option<[Option<Vec<(String, String)>>; 3]> {
    fn render_table(&self) -> Html {
        if let Some(array) = self {
            html! {
                <>
                    {
                        array.iter().enumerate().map(|(i, x)| 
                            if let Some(vec) = x {
                                html! {
                                    <tr>
                                        <th> {&format!("Rank {}", i)} </th>
                                        {vec.iter().map(|(n, h)| html! {
                                            <>
                                                <td>{&n} </td>
                                                <td>{&h} </td>
                                            </>
                                        }).collect::<Html>()}
                                    </tr>
                                }
                            }
                            else {
                                html! {
                                    <>
                                    </>
                                }
                            }
                        ).collect::<Html>()
                    }
                </>
            }
        } else {
            html! {
                <>
                </>
            }
        }
    }
    fn render_csv(&self) -> Html {
        html! {
            <p class="text-left" id="csv-winners">
                {"place,name,hash"}
                {
                    if let Some(array) = self {
                        html! {
                            <>
                                {if let Some(vec) = &array[0] {
                                    vec.iter().map(|p| render_single(0, &p.0, &p.1)).collect::<Html>()
                                }
                                else {
                                    html! {<></>}
                                }}
                                {if let Some(vec) = &array[1] {
                                    vec.iter().map(|p| render_single(1, &p.0, &p.1)).collect::<Html>()
                                }
                                else {
                                    html! {<></>}
                                }}
                                {if let Some(vec) = &array[2] {
                                    vec.iter().map(|p| render_single(2, &p.0, &p.1)).collect::<Html>()
                                }
                                else {
                                    html! {<></>}
                                }}
                            </>
                        }
                        
                    }
                    else {
                        html! {<></>}
                    }
                }
            </p>
        }
    }
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
    matrix: Option<(f64, f64, f64, f64, f64, f64)>,
    dragging: bool,
    is_consumer_target: bool,
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
            matrix: None,
            dragging: false,
            is_consumer_target: true,
        }
    }
    fn data(&mut self, trending: u8, supply_shock: u8, subsidies: u8) {
        self.trending = trending;
        self.supply_shock = supply_shock;
        self.subsidies = subsidies;
        self.consumer_move(self.consumer_x, self.consumer_y);
        self.producer_move(self.producer_x, self.producer_y);
    }
    fn reset_matrix(&mut self) {
        self.matrix = None;
    }
    fn consumer_move(&mut self, mouse_x: f64, mouse_y: f64) {
        let extra_y = self.trending;
        // ConsoleService::log(&format!("Mouse_x: {}, mouse_y: {}", mouse_x, mouse_y));
        let t = Graphs::get_closest_point_to_cubic_bezier(
            10,
            mouse_x,
            mouse_y,
            0.,
            1.,
            20,
            0.,
            (extra_y + 80).into(),
            40.,
            (extra_y + 80).into(),
            65.,
            (extra_y + 70).into(),
            80.,
            extra_y.into(),
        );
        self.consumer_x = 3. * f64::powi(1. - t, 2) * t * 40.
            + 3. * (1. - t) * f64::powi(t, 2) * 65.
            + f64::powi(t, 3) * 80.;
        self.consumer_y = f64::powi(1. - t, 3) * 80.
            + 3. * f64::powi(1. - t, 2) * t * 80.
            + 3. * (1. - t) * f64::powi(t, 2) * 70.
            + f64::powi(t, 3)
            + f64::from(extra_y);
    }
    fn producer_move(&mut self, mouse_x: f64, mouse_y: f64) {
        // * extra cost
        let extra_y: i16 = i16::from(self.supply_shock) - i16::from(self.subsidies);
        // let extra_y: i16 = -(i16::from(self.supply_shock) - i16::from(self.subsidies));
        // ConsoleService::log(&format!("Mouse_x: {}, mouse_y: {}", mouse_x, mouse_y));
        let t = Graphs::get_closest_point_to_cubic_bezier(
            10,
            mouse_x,
            mouse_y,
            0.,
            1.,
            20,
            0.,
            (extra_y + 80).into(),
            10.,
            (extra_y - 10).into(),
            45.,
            (extra_y - 10).into(),
            80.,
            (extra_y + 100).into(),
        );
        self.producer_x = 3. * f64::powi(1. - t, 2) * t * 10.
            + 3. * (1. - t) * f64::powi(t, 2) * 45.
            + f64::powi(t, 3) * 80.;
        self.producer_y = f64::powi(1. - t, 3) * 80.
            + 3. * f64::powi(1. - t, 2) * t * -10.
            + 3. * (1. - t) * f64::powi(t, 2) * -10.
            + f64::powi(t, 3) * 100.
            + f64::from(extra_y)
        // self.producer_y = f64::powi(1. - t, 3) * f64::from(extra_y + 80)
        //     + 3. * f64::powi(1. - t, 2) * t * f64::from(extra_y - 10)
        //     + 3. * (1. - t) * f64::powi(t, 2) * f64::from(extra_y - 10)
        //     + f64::powi(t, 3) * f64::from(extra_y + 100);
    }
    // * Takes in number of iterations, the point to be projected, the start and end bounds on the guess, the resolution (slices), and the control points
    // * Returns the t value of the minimum
    #[allow(clippy::too_many_arguments)]
    fn get_closest_point_to_cubic_bezier(
        iterations: u32,
        fx: f64,
        fy: f64,
        start: f64,
        end: f64,
        slices: u32,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
    ) -> f64 {
        if iterations == 0 {
            return (start + end) / 2.;
        };
        let tick: f64 = (end - start) / f64::from(slices);
        // let (mut x, mut y);
        let (mut x, mut y);
        let (mut dx, mut dy);
        let mut best: f64 = 0.;
        let mut best_distance: f64 = f64::INFINITY;
        let mut current_distance: f64;
        let mut t: f64 = start;
        while t <= end {
            //B(t) = (1-t)**3 p0 + 3(1 - t)**2 t P1 + 3(1-t)t**2 P2 + t**3 P3
            x = (1. - t) * (1. - t) * (1. - t) * x0
                + 3. * (1. - t) * (1. - t) * t * x1
                + 3. * (1. - t) * t * t * x2
                + t * t * t * x3;
            y = (1. - t) * (1. - t) * (1. - t) * y0
                + 3. * (1. - t) * (1. - t) * t * y1
                + 3. * (1. - t) * t * t * y2
                + t * t * t * y3;
            dx = x - fx;
            dy = y - fy;
            dx = f64::powi(dx, 2);
            dy = f64::powi(dy, 2);
            current_distance = dx + dy;
            if current_distance < best_distance {
                best_distance = current_distance;
                best = t;
            }
            t += tick;
        }
        // ConsoleService::log(&format!(
        //     "Best t: {}, best distance: {}, x: {}, y: {}",
        //     best, best_distance, x, y
        // ));
        Graphs::get_closest_point_to_cubic_bezier(
            iterations - 1,
            fx,
            fy,
            f64::max(best - tick, 0.),
            f64::min(best + tick, 1.),
            slices,
            x0,
            y0,
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        )
    }
}

enum Msg {
    Connect(Vec<String>),                       // connect to websocket server
    Disconnected,                               // disconnected from server
    Ignore,                                     // ignore this message
    Received(Result<DirectorServerMsg, Error>), // data received from server
    PrepWsConnect,
    FailedToConnect,
    EndGame,
    HandleKick(Option<EventTarget>),
    StartClick(yew::MouseEvent, bool),
    MouseMove(yew::MouseEvent),
    StartTouch(yew::TouchEvent, bool),
    TouchMove(yew::TouchEvent),
    EndDrag,
    ToggleOpen,
    AdjustOffset(u8),
    NextTurn,
}

impl Model {
    fn render_buttons(&self) -> Html {
        // * consumer's turn. can change producer offsets
        if self.turn % 2 == 0 {
            html! {
                <>
                    <button onclick={self.link.callback(|_| Msg::AdjustOffset(1))} class="btn btn-primary border rounded" type="button">{format!("Supply Shock: {}", self.graph_data.supply_shock)}</button>
                    <button onclick={self.link.callback(|_| Msg::AdjustOffset(2))} class="btn btn-primary border rounded" type="button">{format!("Subsidies: {}", self.graph_data.subsidies)}</button>
                    <button onclick={self.link.callback(|_| Msg::AdjustOffset(3))} class="btn btn-primary border rounded disabled" type="button" disabled=true>{format!("Trending: {}", self.graph_data.trending)}</button>
                </>
            }
        } else {
            html! {
                <>
                    <button onclick={self.link.callback(|_| Msg::AdjustOffset(1))} class="btn btn-primary border rounded disabled" type="button" disabled=true>{format!("Supply Shock: {}", self.graph_data.supply_shock)}</button>
                    <button onclick={self.link.callback(|_| Msg::AdjustOffset(2))} class="btn btn-primary border rounded disabled" type="button" disabled=true>{format!("Subsidies: {}", self.graph_data.subsidies)}</button>
                    <button onclick={self.link.callback(|_| Msg::AdjustOffset(3))} class="btn btn-primary border rounded" type="button">{format!("Trending: {}", self.graph_data.trending)}</button>
                </>
            }
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            ws: None,
            text: String::new(),
            task: None,
            consumers: HashMap::new(),
            producers: HashMap::new(),
            directors: HashMap::new(),
            viewers: HashMap::new(),
            is_open: "Open".to_string(),
            graph_data: Graphs::new(),
            game_id: "".to_string(),
            personal_id: "".to_string(),
            turn: 0,
            winners: None,
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
                    let window = web_sys::window;
                    let host: String = window().unwrap().location().host().unwrap();
                    let protocol: String = window().unwrap().location().protocol().unwrap();
                    // let url = format!("ws://{}/ws/{}/{}/{}", host, v[0], v[1], v[2]);
                    let url = match protocol.as_str() {
                        "http:" => {
                            format!("ws://{}/ws/{}/{}/{}", host, v[0], v[1], v[2])
                        }
                        "https:" => {
                            format!("wss://{}/ws/{}/{}/{}", host, v[0], v[1], v[2])
                        }
                        &_ => return false,
                    };
                    // let url = format!("wss://{}/ws/{}/{}/{}", host, v[0], v[1], v[2]);
                    self.personal_id = v[2].to_owned();
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
                    DirectorServerType::Info(info) => {
                        if info.is_open {
                            self.is_open = "Close".to_string();
                        } else {
                            self.is_open = "Open".to_string();
                        }
                        self.game_id = info.game_id;
                        self.turn = info.turn;
                        self.graph_data
                            .data(info.trending, info.supply_shock, info.subsidies);
                        self.consumers.extend(info.consumers.into_iter());
                        self.producers.extend(info.producers.into_iter());
                        self.directors.extend(info.directors.into_iter());
                        self.viewers.extend(info.viewers.into_iter());
                    }
                    DirectorServerType::Ping => {
                        if let Some(ref mut task) = self.ws {
                            ConsoleService::log("Sending Pong");
                            task.send_binary(Ok(to_vec(&DirectorClientMsg {
                                msg_type: DirectorClientType::Pong,
                            })
                            .unwrap()));
                        }
                        return false;
                    }
                    DirectorServerType::NewConsumer(id, name) => {
                        self.consumers.insert(name, Participant::new(true, id));
                    }
                    DirectorServerType::NewProducer(id, name) => {
                        self.producers.insert(name, Participant::new(true, id));
                    }
                    DirectorServerType::NewDirector(id, name) => {
                        self.directors.insert(name, Participant::new(false, id));
                    }
                    DirectorServerType::NewViewer(id, name) => {
                        self.viewers.insert(name, Participant::new(false, id));
                    }
                    DirectorServerType::ParticipantKicked(target) => {
                        ConsoleService::log("Received Message to kick: ");
                        ConsoleService::log(&target);
                        //* remove any references to this id
                        self.producers.remove(&target);
                        self.consumers.remove(&target);
                        self.directors.remove(&target);
                        self.viewers.remove(&target);
                    }
                    DirectorServerType::GameOpened => {
                        self.is_open = "Close".to_owned();
                    }
                    DirectorServerType::GameClosed => {
                        self.is_open = "Open".to_owned();
                    }
                    DirectorServerType::NewOffsets(offsets) => {
                        self.graph_data.data(
                            offsets.trending,
                            offsets.supply_shock,
                            offsets.subsidies,
                        );
                    }
                    DirectorServerType::TurnAdvanced => {
                        self.turn += 1;
                        for producer in self.producers.values_mut() {
                            producer.took_turn = Some(false);
                        }
                        for consumer in self.consumers.values_mut() {
                            consumer.took_turn = Some(false);
                        }
                    }
                    DirectorServerType::DisconnectedPlayer(target, participant_type) => {
                        match participant_type.as_str() {
                            "consumer" => self
                                .consumers
                                .update_status(&target, PlayerState::Disconnected),
                            "producer" => self
                                .producers
                                .update_status(&target, PlayerState::Disconnected),
                            "director" => self
                                .directors
                                .update_status(&target, PlayerState::Disconnected),
                            "viewer" => self
                                .viewers
                                .update_status(&target, PlayerState::Disconnected),
                            _ => (),
                        }
                    }
                    DirectorServerType::UnresponsivePlayer(target, participant_type) => {
                        match participant_type.as_str() {
                            "consumer" => self
                                .consumers
                                .update_status(&target, PlayerState::Unresponsive),
                            "producer" => self
                                .producers
                                .update_status(&target, PlayerState::Unresponsive),
                            "director" => self
                                .directors
                                .update_status(&target, PlayerState::Unresponsive),
                            "viewer" => self
                                .viewers
                                .update_status(&target, PlayerState::Unresponsive),
                            _ => (),
                        }
                    }
                    DirectorServerType::ConnectedPlayer(target, participant_type) => {
                        match participant_type.as_str() {
                            "consumer" => self
                                .consumers
                                .update_status(&target, PlayerState::Connected),
                            "producer" => self
                                .producers
                                .update_status(&target, PlayerState::Connected),
                            "director" => self
                                .directors
                                .update_status(&target, PlayerState::Connected),
                            "viewer" => self.viewers.update_status(&target, PlayerState::Connected),
                            _ => (),
                        }
                    }
                    DirectorServerType::ServerKicked => {
                        self.ws = None;
                        js! {
                            document.getElementById("kick-modal").click();
                        }
                    }
                    DirectorServerType::TurnTaken(target, participant_type) => {
                        match participant_type.as_str() {
                            "consumer" => self.consumers.took_turn(&target),
                            "producer" => self.producers.took_turn(&target),
                            _ => (),
                        }
                    }
                    DirectorServerType::GameEnded => {
                        js! {
                            document.getElementById("end-modal").click();
                        }
                    }
                    DirectorServerType::Winners(list) => {
                        self.winners = Some(list);
                        js! {
                            document.getElementById("win-modal").click();
                        }
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
            Msg::EndGame => {
                if let Some(ref mut task) = self.ws {
                    task.send_binary(Ok(to_vec(&DirectorClientMsg {
                        msg_type: DirectorClientType::EndGame,
                    })
                    .unwrap()));
                    true
                } else {
                    false
                }
            }
            Msg::HandleKick(possible_target) => {
                if let Some(ref mut task) = self.ws {
                    if let Some(target) = possible_target {
                        if let Some(element) = target.dyn_ref::<HtmlParagraphElement>() {
                            if element.id() != self.personal_id {
                                match element.class_name().as_ref() {
                                    "kickable live" | "kickable unresponsive" | "kickable" => {
                                        element.set_class_name("kicked");
                                        task.send_binary(Ok(to_vec(&DirectorClientMsg {
                                            msg_type: DirectorClientType::Kick(element.id()),
                                        })
                                        .unwrap()));
                                        return true;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                false
            }
            Msg::ToggleOpen => {
                if let Some(ref mut task) = self.ws {
                    if self.is_open == "Open" {
                        task.send_binary(Ok(to_vec(&DirectorClientMsg {
                            msg_type: DirectorClientType::OpenGame,
                        })
                        .unwrap()));
                    } else {
                        task.send_binary(Ok(to_vec(&DirectorClientMsg {
                            msg_type: DirectorClientType::CloseGame,
                        })
                        .unwrap()));
                    }
                }
                false
            }
            Msg::StartClick(event, is_consumer) => {
                if let Some(target) = event.current_target() {
                    let element: SvggElement = target.dyn_ref::<SvggElement>().unwrap().clone();
                    let matrix: web_sys::SvgMatrix =
                        element.get_screen_ctm().unwrap().inverse().unwrap();
                    self.graph_data.matrix = Some((
                        matrix.a().into(),
                        matrix.b().into(),
                        matrix.c().into(),
                        matrix.d().into(),
                        matrix.e().into(),
                        matrix.f().into(),
                    ));
                    let matrix = self.graph_data.matrix.unwrap();
                    let temp_x: f64 = event.client_x().into();
                    let temp_y: f64 = event.client_y().into();
                    let mouse_x = matrix.0 * temp_x + matrix.2 * temp_y + matrix.4;
                    let mouse_y = matrix.1 * temp_x + matrix.3 * temp_y + matrix.5;
                    if is_consumer {
                        self.graph_data.consumer_move(mouse_x, mouse_y);
                    } else {
                        self.graph_data.producer_move(mouse_x, mouse_y);
                    }
                    self.graph_data.dragging = true;
                    self.graph_data.is_consumer_target = is_consumer;
                }
                true
            }
            Msg::MouseMove(event) => {
                if self.graph_data.dragging {
                    let matrix = self.graph_data.matrix.unwrap();
                    let temp_x: f64 = event.client_x().into();
                    let temp_y: f64 = event.client_y().into();
                    let mouse_x = matrix.0 * temp_x + matrix.2 * temp_y + matrix.4;
                    let mouse_y = matrix.1 * temp_x + matrix.3 * temp_y + matrix.5;
                    if self.graph_data.is_consumer_target {
                        self.graph_data.consumer_move(mouse_x, mouse_y);
                    } else {
                        self.graph_data.producer_move(mouse_x, mouse_y);
                    }
                    true
                } else {
                    false
                }
            }
            Msg::EndDrag => {
                self.graph_data.dragging = false;
                self.graph_data.reset_matrix();
                false
            }
            Msg::StartTouch(event, is_consumer) => {
                let list = event.changed_touches();
                if list.length() == 1 {
                    if let Some(touch) = list.get(0) {
                        let window = web_sys::window().unwrap();
                        let document = window.document().unwrap();
                        let temp_x: f64 = touch.client_x().into();
                        let temp_y: f64 = touch.client_y().into();
                        let string;
                        if is_consumer {
                            string = "Consumer Group";
                        } else {
                            string = "Producer Group";
                        }
                        let element: SvggElement = document
                            .get_element_by_id(string)
                            .unwrap()
                            .dyn_ref::<web_sys::SvggElement>()
                            .unwrap()
                            .clone();
                        let matrix: web_sys::SvgMatrix =
                            element.get_screen_ctm().unwrap().inverse().unwrap();
                        self.graph_data.matrix = Some((
                            matrix.a().into(),
                            matrix.b().into(),
                            matrix.c().into(),
                            matrix.d().into(),
                            matrix.e().into(),
                            matrix.f().into(),
                        ));
                        let matrix = self.graph_data.matrix.unwrap();
                        let mouse_x = matrix.0 * temp_x + matrix.2 * temp_y + matrix.4;
                        let mouse_y = matrix.1 * temp_x + matrix.3 * temp_y + matrix.5;
                        if is_consumer {
                            self.graph_data.consumer_move(mouse_x, mouse_y);
                        } else {
                            self.graph_data.producer_move(mouse_x, mouse_y);
                        }
                        self.graph_data.dragging = true;
                        self.graph_data.is_consumer_target = is_consumer;
                    }
                }
                true
            }
            Msg::TouchMove(event) => {
                if self.graph_data.dragging {
                    let list = event.touches();
                    if list.length() == 1 {
                        if let Some(touch) = list.get(0) {
                            let temp_x: f64 = touch.client_x().into();
                            let temp_y: f64 = touch.client_y().into();
                            let matrix = self.graph_data.matrix.unwrap();
                            let mouse_x = matrix.0 * temp_x + matrix.2 * temp_y + matrix.4;
                            let mouse_y = matrix.1 * temp_x + matrix.3 * temp_y + matrix.5;
                            if self.graph_data.is_consumer_target {
                                self.graph_data.consumer_move(mouse_x, mouse_y);
                            } else {
                                self.graph_data.producer_move(mouse_x, mouse_y);
                            }
                            return true;
                        }
                    } else {
                        self.graph_data.dragging = false;
                    }
                }
                false
            }
            Msg::AdjustOffset(target) => {
                let mut offsets = Offsets {
                    subsidies: self.graph_data.subsidies,
                    trending: self.graph_data.trending,
                    supply_shock: self.graph_data.supply_shock,
                };
                match target {
                    1 => {
                        if self.graph_data.supply_shock == 0 {
                            offsets.supply_shock = 10;
                        } else {
                            offsets.supply_shock = 0;
                        }
                    }
                    2 => {
                        if self.graph_data.subsidies == 0 {
                            offsets.subsidies = 10;
                        } else {
                            offsets.subsidies = 0;
                        }
                    }
                    3 => {
                        if self.graph_data.trending == 0 {
                            offsets.trending = 10;
                        } else {
                            offsets.trending = 0;
                        }
                    }
                    _ => (),
                }
                if let Some(ref mut task) = self.ws {
                    task.send_binary(Ok(to_vec(&DirectorClientMsg {
                        msg_type: DirectorClientType::NewOffsets(offsets),
                    })
                    .unwrap()));
                }
                false
            }
            Msg::NextTurn => {
                if let Some(ref mut task) = self.ws {
                    task.send_binary(Ok(to_vec(&DirectorClientMsg {
                        msg_type: DirectorClientType::NextTurn,
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
        let open_close = self.link.callback(|_| Msg::ToggleOpen);
        let producer_click_down = self.link.callback(|event| Msg::StartClick(event, false));
        let consumer_click_down = self.link.callback(|event| Msg::StartClick(event, true));
        let click_move = self.link.callback(Msg::MouseMove);
        let consumer_touch_start = self.link.callback(|event| Msg::StartTouch(event, true));
        let producer_touch_start = self.link.callback(|event| Msg::StartTouch(event, false));
        let touch_move = self.link.callback(Msg::TouchMove);
        let end_drag = self.link.callback(|_| Msg::EndDrag);
        let handle_click = self
            .link
            .callback(|e: MouseEvent| Msg::HandleKick(e.target()));
        let copy = self.link.callback(|_| {
            js! {
                navigator.clipboard.writeText(document.getElementById("csv-winners").innerText.replace("<br>","\\n"));
            };
            Msg::Ignore
        });

        html! {
            <>
                <div class="container text-center">
                <h1> {"Director Controls"}</h1>
                    <div class="row" style="margin-right: 0;margin-left: 0;">
                        <div class="col-md-4 text-center" style="padding: 0;min-height: 40vmin;">
                            <div class="row">
                                <div class="col" style="min-height: 30vmin;">
                                    <h2>{"Events"}</h2>
                                    <div class="btn-group-vertical btn-group-lg" role="group">
                                        {self.render_buttons()}
                                    </div>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col" style="min-height: 15vmin;">
                                    <h2>{"Control Flow"}</h2>
                                    <button onclick={self.link.callback(|_| Msg::NextTurn)} class="btn btn-lg btn-warning border rounded" type="button">{"Force Next Turn"}</button>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col" style="min-height: 40vmin;">
                                    <h2>{"Danger"}</h2>
                                        <div class="btn-group-vertical btn-group-lg" role="group">
                                        <button onclick=open_close class="btn btn-primary border rounded" type="button">{&self.is_open}</button>
                                        <button class="btn btn-danger border rounded" type="button" data-toggle="modal" data-target="#confirm-modal">{"End Game"}</button>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="col-md-4 text-center" style="padding: 0;min-height: 40vmin;">
                            <div class="d-flex flex-column" style="height: 100%;width: 100%;">
                                <h2>{"Graphs"}</h2>
                                <div class="d-xl-flex flex-fill justify-content-xl-center align-items-xl-center" style="width: 100%">
                                    <svg viewBox="-5 -5 100 100" preserveAspectRatio="xMidYMid meet" fill="white">
                                        <g id="Consumer Group" transform="scale(1,-1) translate(0,-90)" style="cursor:cell" onmousedown=consumer_click_down onmousemove=click_move.clone() onmouseup=end_drag.clone() onmouseleave=end_drag.clone() ontouchstart=consumer_touch_start ontouchmove=touch_move.clone()>
                                            <rect width="105" height="105" x="-5" y="-5" fill-opacity="0%"></rect>
                                            <text x="10" y="-30" style="font: 10px Georgia; " transform="scale(1,-1)">{format!("{:.2}, {:.2}",self.graph_data.consumer_x,self.graph_data.consumer_y)}</text>
                                            <path d="M 0 80 C 40 80, 65 70, 80 0" stroke="#6495ED" stroke-width="1" stroke-opacity="60%" fill-opacity="0%" stroke-dasharray="4" />
                                            <path d={
                                                let temp: i16 = self.graph_data.trending.into();
                                                format!("M 0 {} C 40 {}, 65 {}, 80 {}", temp+80, temp+80, temp+70, temp)
                                            }  stroke="white" stroke-width="1" fill="transparent"/>
                                            <polygon points="0,95 -5,90 -1,90 -1,-1 90,-1 90,-5 95,0 90,5 90,1 1,1 1,90 5,90" fill="#1F6DDE" />
                                            <line x1="25" x2="25" y1="2" y2="-2" stroke="white" stroke-width="1"/>
                                            <line x1="50" x2="50" y1="3" y2="-3" stroke="white" stroke-width="1"/>
                                            <text y="-5" x="47" style="font: 5px Georgia; " transform="scale(1,-1)">{"50"}</text>
                                            <line x1="75" x2="75" y1="2" y2="-2" stroke="white" stroke-width="1"/>
                                            <line y1="25" y2="25" x1="2" x2="-2" stroke="white" stroke-width="1"/>
                                            <line y1="50" y2="50" x1="3" x2="-3" stroke="white" stroke-width="1"/>
                                            <text x="5" y="-49" style="font: 5px Georgia; " transform="scale(1,-1)">{"50"}</text>
                                            <line y1="75" y2="75" x1="2" x2="-2" stroke="white" stroke-width="1"/>
                                            <circle cx={format!("{:.2}",self.graph_data.consumer_x)} cy={format!("{:.2}",self.graph_data.consumer_y)} r="3" stroke="white" fill="#F34547" stroke-width="0.2"/>
                                        </g>
                                    </svg>
                                </div>
                                <div class="d-xl-flex flex-fill justify-content-xl-center align-items-xl-center" style="width: 100%;">
                                    <svg viewBox="-5 -5 100 100" preserveAspectRatio="xMidYMid meet" fill="white">
                                        <g id="Producer Group" transform="scale(1,-1) translate(0,-90)" style="cursor:cell" onmousedown=producer_click_down onmousemove=click_move onmouseup=end_drag.clone() onmouseleave=end_drag.clone() ontouchstart=producer_touch_start ontouchmove=touch_move>
                                            <rect width="105" height="105" x="-5" y="-5" fill-opacity="0%"></rect>
                                            <text x="10" y="-70" style="font: 10px Georgia; " transform="scale(1,-1)">{format!("{:.2}, {:.2}",self.graph_data.producer_x,self.graph_data.producer_y)}</text>
                                            <path d="M 0 80 C 10 -10, 45 -10, 80 100" stroke="#6495ED" stroke-width="1" stroke-opacity="60%" fill-opacity="0%" stroke-dasharray="4" />
                                            <path d={
                                                let net: i16 = i16::from(self.graph_data.supply_shock) - i16::from(self.graph_data.subsidies);
                                                format!("M 0 {} C 10 {}, 45 {}, 80 {}", net+80, net-10, net-10, net+100)
                                            } stroke="white" stroke-width="1" fill="transparent"/>
                                            <polygon points="0,95 -5,90 -1,90 -1,-1 90,-1 90,-5 95,0 90,5 90,1 1,1 1,90 5,90" fill="#1F6DDE" />
                                            <line x1="25" x2="25" y1="2" y2="-2" stroke="white" stroke-width="1"/>
                                            <line x1="50" x2="50" y1="3" y2="-3" stroke="white" stroke-width="1"/>
                                            <text y="-5" x="47" style="font: 5px Georgia; " transform="scale(1,-1)">{"50"}</text>
                                            <line x1="75" x2="75" y1="2" y2="-2" stroke="white" stroke-width="1"/>
                                            <line y1="25" y2="25" x1="2" x2="-2" stroke="white" stroke-width="1"/>
                                            <line y1="50" y2="50" x1="3" x2="-3" stroke="white" stroke-width="1"/>
                                            <line y1="75" y2="75" x1="2" x2="-2" stroke="white" stroke-width="1"/>
                                            <circle cx={format!("{:.2}",self.graph_data.producer_x)} cy={format!("{:.2}",self.graph_data.producer_y)} r="3" stroke="white" fill="#F34547" stroke-width="0.2"/>
                                        </g>
                                    </svg>
                                </div>
                            </div>
                        </div>
                        <div class="col-md-4 text-center" style="padding: 0;min-height: 40vmin;">
                            <h2>{"State"}</h2>
                            <p>{format!("Game ID: {}", self.game_id)}</p>
                            <p>{format!("Turn: {}", self.turn)}</p>
                            <div onclick=handle_click id="participants" style="overflow-y: scroll;max-height: 50vh;">
                                <p class="lead" style="background: var(--dark);">{"Directors"}</p>
                                    {self.directors.render()}
                                    <p class="lead" style="background: var(--dark);">{"Viewers"}</p>
                                    {self.viewers.render()}
                                    <p class="lead" style="background: var(--dark);">{"Consumers"}</p>
                                    {self.consumers.render()}
                                    <p class="lead" style="background: var(--dark);">{"Producers"}</p>
                                    {self.producers.render()}
                            </div>
                        </div>
                    </div>
                    <footer>
                        <p>{"Built by Francis Chua"}</p>
                    </footer>
                    <div class="modal fade" role="dialog" tabindex="-1" id="confirm-modal">
                        <div class="modal-dialog" role="document">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h4 class="modal-title">{"Confirm End Game"}</h4><button type="button" class="close" data-dismiss="modal" aria-label="Close"><span aria-hidden="true">{"×"}</span></button>
                                </div>
                                <div class="modal-body">
                                    <p>{"Are you sure you want to end this game for all participants?"}</p>
                                </div>
                                <div class="modal-footer"><
                                    button class="btn btn-light" type="button" data-dismiss="modal">{"No"}</button>
                                    <button onclick=self.link.callback(|_| Msg::EndGame) class="btn btn-danger" type="button" data-dismiss="modal">{"Yes"}</button>
                                </div>
                            </div>
                        </div>
                    </div>
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
                    <button class="btn btn-danger border rounded" id="end-modal" type="button" data-toggle="modal" data-target="#ended-modal" hidden=true></button>
                    <div role="dialog" tabindex="-1" class="modal fade" id="ended-modal">
                        <div class="modal-dialog" role="document">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h4 class="modal-title">{"Game Ended"}</h4><button type="button" class="close" data-dismiss="modal" aria-label="Close"><span aria-hidden="true">{"×"}</span></button>
                                </div>
                                <div class="modal-body">
                                    <p>{"This game has been ended by a Director."}</p>
                                </div>
                                <div class="modal-footer">
                                    <a class="btn btn-info active" role="button" href="/login/index.html">{"Continue to Login"}</a>
                                </div>
                            </div>
                        </div>
                    </div>
                    <button class="btn btn-danger border rounded" id="win-modal" type="button" data-toggle="modal" data-target="#winner-modal" hidden=true></button>
                    <div class="modal fade" role="dialog" tabindex="-1" id="winner-modal">
                        <div class="modal-dialog" role="document">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h4 class="modal-title">{"Winners"}</h4><button type="button" class="close" data-dismiss="modal" aria-label="Close"><span aria-hidden="true">{"×"}</span></button>
                                </div>
                                <div class="modal-body">
                                    <div>
                                        <ul class="nav nav-tabs" role="tablist">
                                            <li class="nav-item" role="presentation"><a class="nav-link active" role="tab" data-toggle="tab" href="#tab-2">{"CSV"}</a></li>
                                            <li class="nav-item" role="presentation"><a class="nav-link" role="tab" data-toggle="tab" href="#tab-1">{"Table"}</a></li>
                                        </ul>
                                        <div class="tab-content" style="max-height: 60vh; overflow-y: auto;">
                                            <div class="tab-pane" role="tabpanel" id="tab-1">
                                                <div class="table-responsive text-nowrap">
                                                    <table class="table table-hover table-dark">
                                                        <tbody>
                                                            {self.winners.render_table()}
                                                        </tbody>
                                                    </table>
                                                </div>
                                            </div>
                                            <div class="tab-pane active" role="tabpanel" id="tab-2">
                                                {self.winners.render_csv()}
                                                <button class="btn btn-success active btn-block pulse animated" id="copy-button" type="button" onclick=copy>{"Copy"}</button>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="modal-footer"><a class="btn btn-info active" role="button" href="/login/index.html">{"Continue to Login"}</a></div>
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
