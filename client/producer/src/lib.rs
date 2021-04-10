#![recursion_limit = "2048"]
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::SvggElement;
use yew::prelude::*;

extern crate console_error_panic_hook;
use anyhow::Error;
use std::panic;

use http::{Request, Response};
use yew::html::ComponentLink;
use yew::services::fetch;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;

use serde_cbor::{from_slice, to_vec};

use stdweb::js;

mod structs;
use structs::{
    ClientExtraFields, ProducerClientMsg, ProducerClientType, ProducerServerMsg,
    ProducerServerType, /* Offsets,*/
};

use structs::Participant;

struct Model {
    link: ComponentLink<Self>,
    ws: Option<WebSocketTask>,
    // server_data: String, // data received from the server
    fetch_task: Option<fetch::FetchTask>,
    // client_data: DirectorClientMsg,
    producers: HashMap<String, Participant>,
    graph_data: Graphs,
    turn: u64,
    game_id: String,
    quantity: f64,
    price: f64,
    score: f64,
    balance: f64,
    took_turn: bool,
    error_msg: String,
    // personal_id: String,
}

impl Participant {
    fn render(&self, id: String) -> Html {
        html! {
            <div class="seller">
                <p class="d-flex flex-grow-1 id" style="margin-bottom: 0px;">{&id}<br/></p>
                <p class="text-center d-xl-flex flex-grow-1 quantity" style="margin-bottom: 0px;">{format!("${:.2}",self.price)}</p>
                <p class="text-center d-xl-flex flex-grow-1 quantity" style="margin-bottom: 0px;">{format!("{}/{}",self.remaining,self.produced)}</p>
            </div>
        }
    }
}

trait ParticipantCollection {
    fn render(&self) -> Html;
}

impl ParticipantCollection for HashMap<String, Participant> {
    fn render(&self) -> Html {
        html! {
            <>
                {for self.keys().zip(self.values()).map(|tuple| tuple.1.render(tuple.0.to_string()))}
            </>
        }
    }
}

// ! Make sure allowed graph values never goes below 0.
struct Graphs {
    producer_x: f64,
    producer_y: f64,
    supply_shock: u8,
    subsidies: u8,
    matrix: Option<(f64, f64, f64, f64, f64, f64)>,
    dragging: bool,
}

impl Graphs {
    fn new() -> Graphs {
        Graphs {
            producer_x: 32.50,
            producer_y: 15.00,
            supply_shock: 0,
            subsidies: 0,
            matrix: None,
            dragging: false,
        }
    }
    fn data(&mut self, supply_shock: u8, subsidies: u8) {
        self.supply_shock = supply_shock;
        self.subsidies = subsidies;
        self.producer_move(self.producer_x, self.producer_y);
    }
    fn reset_matrix(&mut self) {
        self.matrix = None;
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
            + f64::from(extra_y);
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
    fn get_t_for_quantity(&self, t_0: f64, t_2: f64, x: f64, iterations: u32) -> f64 {
        if iterations == 0 {
            return t_0;
        }
        let t_1 = (t_0+t_2)/2.;
        let x_1 = 3. * f64::powi(1. - t_1, 2) * t_1 * 10.
        + 3. * (1. - t_1) * f64::powi(t_1, 2) * 45.
        + f64::powi(t_1, 3) * 80. - x;
        if x_1 > 0. {
            self.get_t_for_quantity(t_0, t_1, x, iterations - 1)
        }
        else if x_1 < 0. {
            self.get_t_for_quantity(t_1, t_2, x, iterations - 1)
        }
        else {
            t_1
        }
    }
}

enum Msg {
    Connect(Vec<String>),                       // connect to websocket server
    Disconnected,                               // disconnected from server
    Ignore,                                     // ignore this message
    Received(Result<ProducerServerMsg, Error>), // data received from server
    PrepWsConnect,
    // EndGame,
    // HandleKick(Option<EventTarget>),
    StartClick(yew::MouseEvent),
    MouseMove(yew::MouseEvent),
    StartTouch(yew::TouchEvent),
    TouchMove(yew::TouchEvent),
    EndDrag,
    Quantity(yew::InputData),
    Price(yew::html::InputData),
    Submit,
    // ToggleOpen,
    // AdjustOffset(u8),
    // NextTurn,
}

impl Model {
    fn update_producers(&mut self, new_list: Vec<(String, Participant)>) {
        self.producers.clear();
        for producer in new_list {
            self.producers.insert(producer.0, producer.1);
        }
    }
    fn render_submit(&self) -> Html {
        if self.took_turn || self.turn%2 == 0 {
            html! {<button onclick=self.link.callback(|_| Msg::Submit) class="btn btn-danger disabled btn-block flex-grow-0 flex-shrink-1" type="submit" disabled=true>{"Submit and End Turn"}</button>}
        }
        else {
            html! {<button onclick=self.link.callback(|_| Msg::Submit) class="btn btn-danger btn-block flex-grow-0 flex-shrink-1" type="submit">{"Submit and End Turn"}</button>}
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
            fetch_task: None,
            producers: HashMap::new(),
            graph_data: Graphs::new(),
            game_id: "".to_string(),
            turn: 0,
            price: 0.,
            balance: 0.,
            score: 0.,
            quantity: 0.,
            took_turn: false,
            error_msg: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connect(v) => {
                ConsoleService::log("Connecting");
                let cbout = self.link.callback(|data: Result<Vec<u8>, anyhow::Error>| {
                    Msg::Received(Ok(from_slice::<ProducerServerMsg>(&data.unwrap()).unwrap()))
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
                    ProducerServerType::Info(info) => {
                        ConsoleService::log(&format!("{:?}",info));
                        self.producers.extend(info.producers.into_iter());
                        self.game_id = info.game_id;
                        self.turn = info.turn;
                        self.graph_data.data(info.supply_shock, info.subsidies);
                        self.balance = info.balance;
                        self.score = info.score;
                        self.took_turn = info.took_turn;
                    }
                    ProducerServerType::Ping => {
                        if let Some(ref mut task) = self.ws {
                            ConsoleService::log("Sending Pong");
                            task.send_binary(Ok(to_vec(&ProducerClientMsg {
                                msg_type: ProducerClientType::Pong,
                            })
                            .unwrap()));
                        }
                        return false;
                    }
                    ProducerServerType::ServerKicked => {
                        self.ws = None;
                        js! {
                            document.getElementById("kick-modal").click();
                        }
                    }
                    ProducerServerType::ChoiceFailed(fail_info) => {
                        self.error_msg = fail_info;
                    }
                    ProducerServerType::ChoiceSubmitted(a, b) => {
                        self.score = a; self.balance = b;
                        self.took_turn = true;
                        self.error_msg = "".to_string();
                    }
                    ProducerServerType::NewOffsets(offsets) => {
                        self.graph_data.data(
                            offsets.supply_shock,
                            offsets.subsidies,
                        );
                    }
                    ProducerServerType::TurnInfo(turn_info) => {
                        self.update_producers(turn_info.producers);
                    }
                    ProducerServerType::TurnAdvanced(balance) => {
                        self.turn += 1;
                        self.took_turn = false;
                        if self.turn%2 == 1 {
                            self.balance = balance;
                        }
                    }
                    ProducerServerType::StockReduced(targets) => {
                        for target in targets {
                            if let Some(producer) = self.producers.get_mut(&target.0) {
                                producer.remaining -= &target.1;
                            }
                        }
                    }
                    _ => {}
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
            Msg::StartClick(event) => {
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
                    self.graph_data.producer_move(mouse_x, mouse_y);
                    self.graph_data.dragging = true;
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
                    self.graph_data.producer_move(mouse_x, mouse_y);
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
            Msg::StartTouch(event) => {
                let list = event.changed_touches();
                if list.length() == 1 {
                    if let Some(touch) = list.get(0) {
                        let window = web_sys::window().unwrap();
                        let document = window.document().unwrap();
                        let temp_x: f64 = touch.client_x().into();
                        let temp_y: f64 = touch.client_y().into();
                        let string = "Producer Group";
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
                        self.graph_data.producer_move(mouse_x, mouse_y);
                        self.graph_data.dragging = true;
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
                            self.graph_data.producer_move(mouse_x, mouse_y);
                            return true;
                        }
                    } else {
                        self.graph_data.dragging = false;
                    }
                }
                false
            }
            Msg::Quantity(data) => {
                if let Ok(value) = data.value.parse() {
                    self.quantity = value;
                }
                false
            }
            Msg::Price(data) => {
                if let Ok(value) = data.value.parse() {
                    self.price = value;
                }
                false
            }
            Msg::Submit => {
                if let Some(ref mut task) = self.ws {
                    ConsoleService::log("Sending Submit");
                    let choice = ClientExtraFields {
                        quantity: self.quantity,
                        price: self.price,
                        t: self.graph_data.get_t_for_quantity(0., 1., self.quantity, 50),
                    };
                    task.send_binary(Ok(to_vec(&ProducerClientMsg {
                        msg_type: ProducerClientType::Choice(choice),
                    })
                    .unwrap()));
                }
                
                false
            },
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
        let producer_click_down = self.link.callback(Msg::StartClick);
        let click_move = self.link.callback(Msg::MouseMove);
        let producer_touch_start = self.link.callback(Msg::StartTouch);
        let touch_move = self.link.callback(Msg::TouchMove);
        let end_drag = self.link.callback(|_: yew::MouseEvent| Msg::EndDrag);
        let change_quantity = self.link.callback(Msg::Quantity);
        let change_price = self.link.callback(Msg::Price);

        html! {
            <>
                <div class="container text-center">
                    <h1>{"Producer"}</h1>
                    <div class="row" style="margin-right: 0;margin-left: 0;">
                        <div class="col-md-6 text-center" style="padding: 0;min-height: 40vmin;">
                            <div class="d-flex flex-column" style="height: 100%;width: 100%;">
                                <h2>{"Marginal Cost"}</h2>
                                <div class="d-xl-flex flex-fill justify-content-xl-center align-items-xl-center" style="width: 100%;">
                                    <svg viewBox="-5 -5 100 100" preserveAspectRatio="xMidYMid meet" fill="white" >
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
                                <div class="d-flex">
                                    <p class="text-center text-light mb-auto text-info" style="width: 50%;">{format!("Balance: {}", self.balance)}</p>
                                    <p class="text-center text-light mb-auto text-info" style="width: 50%;">{format!("Score: {:.2}", self.score)}</p>
                                </div>
                            </div>
                        </div>
                        <div class="col-md-6 text-center d-flex flex-column" style="padding: 0;min-height: 40vmin;">
                            <h2>{"Marketplace"}</h2>
                            <p>{format!("Game ID: {}", self.game_id)}</p>
                            <p>{format!("Turn: {}", self.turn)}</p>
                            <div class="d-flex flex-column flex-grow-1 marketplace">
                                <h4>{"Previous Turn"}</h4>
                                <div class="flex-grow-1 flex-shrink-1 sellers" style="background: var(--dark);border-radius: 6px;border: 2px solid var(--secondary);margin: 8px;">
                                    {self.producers.render()}
                                </div>
                            </div>
                            <form>
                                <div class="form-group" style="width: 45%;"><label for="Quantity" style="width: 40%;">{"Quantity"}</label><input oninput=change_quantity class="form-control" type="number" style="width: 60%;background: var(--secondary);color: var(--white);text-align: center;" placeholder="0" min="0" max="80"/></div>
                                <div class="form-group" style="width: 45%;"><label for="Price"    style="width: 40%;">{"Price"}   </label><input oninput=change_price class="form-control" type="number" style="width: 60%;color: var(--white);background: var(--secondary);text-align: center;" placeholder="0" min="0" max="100"/></div>
                            </form>
                            <div class="d-flex">
                                <p class="text-center text-danger mb-auto text-info" style="width: 100%;">{&self.error_msg}</p>
                            </div>
                            {
                                self.render_submit()
                            }
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
                                <a class="btn btn-info active" role="button" href="/login/index.html" id="test">{"Continue"}</a>
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
