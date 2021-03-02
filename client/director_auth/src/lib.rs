#![recursion_limit = "2048"]
use std::collections::HashMap;
use std::convert::TryInto;
use stdweb::web::document;
use stdweb::web::INonElementParentNode;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::EventTarget;
use web_sys::{HtmlParagraphElement, SvggElement};
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
    consumers: HashMap<String, Participant>,
    producers: HashMap<String, Participant>,
    directors: HashMap<String, Participant>,
    viewers: HashMap<String, Participant>,
    is_open: String,
    graph_data: Graphs,
    // dragging: bool,
}

struct Participant {
    state: PlayerState,
    took_turn: Option<bool>,
}

impl Participant {
    fn new() -> Participant {
        Participant {
            state: PlayerState::Disconnected,
            took_turn: None,
        }
    }
    fn render(&self, id: String) -> Html {
        match self.state {
            PlayerState::Unresponsive => {
                html! {
                    <p class="kickable unresponsive">{id.clone()} <i class="fa fa-signal"></i> {if let Some(turn) = self.took_turn {html! {<i class="fa fa-check"></i>}} else {html!{<i class="fa fa-remove"></i>}}}</p>
                }
            }
            PlayerState::Connected => {
                html! {
                    // <p class="kickable live">{id.clone()}</p>
                    <p class="kickable live">{id.clone()} <i class="fa fa-user"></i> {if let Some(turn) = self.took_turn {html! {<i class="fa fa-check"></i>}} else {html!{<i class="fa fa-remove"></i>}}}</p>
                }
            }
            PlayerState::Disconnected => {
                html! {
                    <p class="kickable">{id.clone()} <i class="fa fa-user-o"></i> {if let Some(turn) = self.took_turn {html! {<i class="fa fa-check"></i>}} else {html!{<i class="fa fa-remove"></i>}}}</p>
                }
            }
            PlayerState::Kicked => {
                html! {
                    <p class="kicked">{id.clone()}</p>
                }
            }
        }
    }
}

trait RenderableCollection {
    fn render(&self) -> Html;
}

impl RenderableCollection for HashMap<String,Participant> {
    fn render(&self) -> Html {
        let iter = self.keys().zip(self.values());
        html! {
            <>
                {for self.keys().zip(self.values()).map(|tuple| tuple.1.render(tuple.0.to_string()))}

            </>
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
            70.,
            (extra_y + 70).into(),
            80.,
            extra_y.into(),
        );
        self.consumer_x = 3. * f64::powi(1. - t, 2) * t * 40.
            + 3. * (1. - t) * f64::powi(t, 2) * 70.
            + f64::powi(t, 3) * 80.;
        self.consumer_y = f64::powi(1. - t, 3) * f64::from(extra_y + 80)
            + 3. * f64::powi(1. - t, 2) * t * f64::from(extra_y + 80)
            + 3. * (1. - t) * f64::powi(t, 2) * f64::from(extra_y + 70)
            + f64::powi(t, 3) * f64::from(extra_y);
    }
    fn producer_move(&mut self, mouse_x: f64, mouse_y: f64) {
        // * extra cost
        let extra_y: i16 = i16::from(self.supply_shock) - i16::from(self.subsidies);
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
            50.,
            (extra_y - 10).into(),
            80.,
            (extra_y + 100).into(),
        );
        self.producer_x = 3. * f64::powi(1. - t, 2) * t * 10.
            + 3. * (1. - t) * f64::powi(t, 2) * 50.
            + f64::powi(t, 3) * 80.;
        self.producer_y = f64::powi(1. - t, 3) * f64::from(extra_y + 80)
            + 3. * f64::powi(1. - t, 2) * t * f64::from(extra_y - 10)
            + 3. * (1. - t) * f64::powi(t, 2) * f64::from(extra_y - 10)
            + f64::powi(t, 3) * f64::from(extra_y + 100);
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
    SendReq,
    PrepWsConnect,
    FailedToConnect,
    EndGame,
    HandleClick(Option<EventTarget>),
    StartClick(yew::MouseEvent, bool),
    MouseMove(yew::MouseEvent),
    StartTouch(yew::TouchEvent, bool),
    TouchMove(yew::TouchEvent),
    EndDrag,
    ToggleOpen,
}

impl Model {}

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
            consumers: HashMap::new(),
            producers: HashMap::new(),
            directors: HashMap::new(),
            viewers: HashMap::new(),
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
                    let url = format!("ws://{}/ws/{}/{}/{}", host, v[0], v[1], v[2]);
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
                        self.consumers.insert(
                            s.target.clone().unwrap(),
                            Participant::new(),
                        );
                    }
                    DirectorServerType::NewProducer => {
                        self.producers.insert(
                            s.target.clone().unwrap(),
                            Participant::new(),
                        );
                    }
                    DirectorServerType::NewDirector => {
                        self.directors.insert(
                            s.target.clone().unwrap(),
                            Participant::new(),
                        );
                    }
                    DirectorServerType::NewViewer => {
                        self.viewers.insert(
                            s.target.clone().unwrap(),
                            Participant::new(),
                        );
                    }
                    DirectorServerType::ParticipantKicked => {
                        ConsoleService::log("Received Message to kick: ");
                        ConsoleService::log(&s.target.clone().unwrap());
                        //* remove any references to this id
                        self.producers.remove(s.target.as_ref().unwrap());
                        self.consumers.remove(s.target.as_ref().unwrap());
                        self.directors.remove(s.target.as_ref().unwrap());
                        self.viewers.remove(s.target.as_ref().unwrap());
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
                match self.ws {
                    Some(ref mut task) => {
                        if let Some(target) = possible_target {
                            if let Some(element) = target.dyn_ref::<HtmlParagraphElement>() {
                                match element.class_name().as_ref() {
                                    "kickable live" | "kickable unresponsive" | "kickable" => {
                                        // element.set_class_name("kicked");
                                        self.consumers.remove(&element.inner_html());
                                        self.producers.remove(&element.inner_html());
                                        self.directors.remove(&element.inner_html());
                                        self.viewers.remove(&element.inner_html());
                                        task.send_binary(Ok(to_vec(&DirectorClientMsg {
                                            msg_type: DirectorClientType::Kick,
                                            kick_target: Some(element.inner_html()),
                                        })
                                        .unwrap()));
                                        return true;
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
                    self.ws
                        .as_mut()
                        .expect("No websocket open")
                        .send_binary(Ok(to_vec(&DirectorClientMsg {
                            msg_type: DirectorClientType::OpenGame,
                            kick_target: None,
                        })
                        .unwrap()));
                } else {
                    self.ws
                        .as_mut()
                        .expect("No websocket open")
                        .send_binary(Ok(to_vec(&DirectorClientMsg {
                            msg_type: DirectorClientType::CloseGame,
                            kick_target: None,
                        })
                        .unwrap()));
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
                    // if let Some(target) = event.current_target() {
                    // let element: SvggElement = target.dyn_ref::<SvggElement>().unwrap().clone();
                    let matrix =
                            // element.get_screen_ctm().unwrap().inverse().unwrap();
                            self.graph_data.matrix.unwrap();
                    let temp_x: f64 = event.client_x().into();
                    let temp_y: f64 = event.client_y().into();
                    let mouse_x = matrix.0 * temp_x + matrix.2 * temp_y + matrix.4;
                    let mouse_y = matrix.1 * temp_x + matrix.3 * temp_y + matrix.5;
                    if self.graph_data.is_consumer_target {
                        self.graph_data.consumer_move(mouse_x, mouse_y);
                    } else {
                        self.graph_data.producer_move(mouse_x, mouse_y);
                    }
                    // }
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
        let svg_consumer_down = self.link.callback(|event| Msg::StartClick(event, true));
        let svg_consumer_move = self.link.callback(Msg::MouseMove);
        let svg_producer_down = self.link.callback(|event| Msg::StartClick(event, false));
        let svg_producer_move = self.link.callback(Msg::MouseMove);
        let end_drag = self.link.callback(|_| Msg::EndDrag);
        let consumertouchstart = self.link.callback(|event| Msg::StartTouch(event, true));
        let producertouchstart = self.link.callback(|event| Msg::StartTouch(event, false));
        let touch_move = self.link.callback(Msg::TouchMove);
        let handle_click = self
            .link
            .callback(|e: MouseEvent| Msg::HandleClick(e.target()));
        html! {
            <>
                <div class="container text-center">
                <h1> {"Director Controls"}</h1>
                    <div class="row" style="margin-right: 0;margin-left: 0;">
                        <div class="col-md-4 text-center" style="padding: 0;min-height: 40vmin;">
                            <div class="row">
                                <div class="col" style="min-height: 30vmin;">
                                    <h2>{"Events"}</h2>
                                    <div class="btn-group-vertical btn-group-lg" role="group"><button class="btn btn-primary border rounded" type="button">{"Supply Shock"}</button><button class="btn btn-primary border rounded" type="button">{"Subsidies"}</button><button class="btn btn-primary border rounded" type="button">{"Trending"}</button></div>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col" style="min-height: 15vmin;">
                                    <h2>{"Control Flow"}</h2><button class="btn btn-lg btn-warning border rounded" type="button">{"Force Next Turn"}</button>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col" style="min-height: 40vmin;">
                                    <h2>{"Danger"}</h2>
                                    <div class="btn-group-vertical btn-group-lg" role="group"><button onclick=open_close class="btn btn-primary border rounded" type="button">{&self.is_open}</button><button class="btn btn-danger border rounded" type="button">{"End Game"}</button></div>
                                </div>
                            </div>
                        </div>
                        <div class="col-md-4 text-center" style="padding: 0;min-height: 40vmin;">
                            <div class="d-flex flex-column" style="height: 100%;width: 100%;">
                                <h2>{"Graphs"}</h2>
                                <div class="d-xl-flex flex-fill justify-content-xl-center align-items-xl-center" style="width: 100%">
                                    <svg viewBox="-5 -5 100 100" preserveAspectRatio="xMidYMid meet" fill="white">
                                        <g id="Consumer Group" transform="scale(1,-1) translate(0,-90)" style="cursor:cell" onmousedown=svg_consumer_down onmousemove=svg_consumer_move onmouseup=end_drag.clone() onmouseleave=end_drag.clone() ontouchstart=consumertouchstart ontouchmove=touch_move.clone()>
                                            <rect width="105" height="105" x="-5" y="-5" fill-opacity="0%"></rect>
                                            <text x="10" y="-30" style="font: 10px Georgia; " transform="scale(1,-1)">{format!("{:.2}, {:.2}",self.graph_data.consumer_x,self.graph_data.consumer_y)}</text>
                                            <path d={
                                                let temp: i16 = self.graph_data.trending.into();
                                                format!("M 0 {} C 40 {}, 70 {}, 80 {}", temp+80, temp+80, temp+70, temp)
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
                                        <g id="Producer Group" transform="scale(1,-1) translate(0,-90)" style="cursor:cell" onmousedown=svg_producer_down onmousemove=svg_producer_move onmouseup=end_drag.clone() onmouseleave=end_drag.clone() ontouchstart=producertouchstart ontouchmove=touch_move>
                                            <rect width="105" height="105" x="-5" y="-5" fill-opacity="0%"></rect>
                                            <text x="10" y="-70" style="font: 10px Georgia; " transform="scale(1,-1)">{format!("{:.2}, {:.2}",self.graph_data.producer_x,self.graph_data.producer_y)}</text>
                                            <path d={
                                                let net: i16 = i16::from(self.graph_data.subsidies) - i16::from(self.graph_data.supply_shock);
                                                format!("M 0 {} C 10 {}, 50 {}, 80 {}", net+80, net-10, net-10, net+100)
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
                            <p>{"Game ID: 123456"}</p>
                            <p>{"Turn: 5"}</p>
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
                </div>
            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
