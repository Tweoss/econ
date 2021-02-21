#![recursion_limit = "1024"]
use wasm_bindgen::prelude::*;
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

// use serde::{Deserialize, Serialize};
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
}

enum Msg {
    Connect(Vec<String>),          // connect to websocket server
    Disconnected,                  // disconnected from server
    Ignore,                        // ignore this message
    TextInput(String),             // text was input in the input box
    SendText,                      // send our text to server
    Received(Result<DirectorServerMsg, Error>), // data received from server
    SendReq,
    PrepWsConnect,
    FailedToConnect,
    EndGame,
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
                    let url = format!("ws://127.0.0.1:8080/ws/{}/{}/{}", v[0], v[1], v[2]);
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
            Msg::TextInput(_e) => {
                // self.client_data.string = e; // note input box value
                false
            }
            Msg::SendText => {
                // match self.ws {
                //     Some(ref mut task) => {
                //         task.send(Json(&self.text));
                //         self.text = "".to_string();
                //         true // clear input box
                //     }
                //     None => false,
                // }
                false
            }
            Msg::Received(Ok(s)) => {
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
            },
            Msg::FailedToConnect => {
                self.text.push_str("Failed to reach /wsprep");
                true
            }
            Msg::EndGame => match self.ws {
                Some(ref mut task) => {
                    task.send_binary(Ok(to_vec(&DirectorClientMsg {msg_type: DirectorClientType::EndGame, kick_target: None}).unwrap()));
                    true
                }
                None => false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let onbuttonconnect = self.link.callback(|_| Msg::PrepWsConnect);
        let onbuttonsend = self.link.callback(|_| Msg::SendText);
        let inputtext = self.link.callback(|e: InputData| Msg::TextInput(e.value));
        let sendreq = self.link.callback(|_| Msg::SendReq);
        let endgame = self.link.callback(|_| Msg::EndGame);
        html! {
            <>
                // <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                // <p>{ self.value }</p>
                <p><button onclick=onbuttonconnect,>{ "Connect" }</button></p><br/>
                // text showing whether we're connected or not
                <p>{ "Connected: " } { !self.ws.is_none() } </p><br/>
                // input box for sending text
                <p><input type="text", value=&self.text, oninput=inputtext,/></p><br/>
                // button for sending text
                <p><button onclick=onbuttonsend,>{ "Send" }</button></p><br/>
                // text area for showing data from the server
                <p><textarea value=&self.server_data,></textarea></p><br/>
                // button to send request
                <p><button onclick=sendreq,>{ "Send Req" }</button></p><br/>
                // close game
                <button onclick=endgame>{ "End Game"}</button>
            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
