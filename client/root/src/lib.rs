#![recursion_limit = "1024"]
use wasm_bindgen::prelude::*;

// #[macro_use]

// #[macro_use]
extern crate failure;
extern crate yew;

// use failure::Error;
use anyhow::Error;

use http::{Request, Response};
use yew::format::{Json, Nothing};
use yew::html::ComponentLink;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::services::ConsoleService;
use yew::services::fetch;

struct Model {
	// console: ConsoleService,
	ws: Option<WebSocketTask>,
	// wss: WebSocketService,
	link: ComponentLink<Model>,
	text: String,        // text in our input box
	server_data: String, // data received from the server
}

enum Msg {
	Connect,                         // connect to websocket server
	Disconnected,                    // disconnected from server
	Ignore,                          // ignore this message
	TextInput(String),               // text was input in the input box
	SendText,                        // send our text to server
	Received(Result<String, Error>), // data received from server
	SendReq,
}

impl Component for Model {
	type Message = Msg;
	type Properties = ();

	fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
		Model {
			// console: ConsoleService {},
			ws: None,
			// wss: WebSocketService {},
			link,
			text: String::new(),
			server_data: String::new(),
		}
	}

	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		match msg {
			Msg::Connect => {
				ConsoleService::log("Connecting");
				let cbout = self.link.callback(|Json(data)| Msg::Received(data));
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
					// let task = match WebSocketService::connect("ws://127.0.0.1:8080/ws", cbout, cbnot) {
					let task =
						match WebSocketService::connect("wss://web.valour.vision/ws", cbout, cbnot)
						{
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
				true
			}
			Msg::Ignore => false,
			Msg::TextInput(e) => {
				self.text = e; // note input box value
				true
			}
			Msg::SendText => {
				match self.ws {
					Some(ref mut task) => {
						task.send(Json(&self.text));
						self.text = "".to_string();
						true // clear input box
					}
					None => false,
				}
			}
			Msg::Received(Ok(s)) => {
				self.server_data.push_str(&format!("{}\n", &s));
				true
			}
			Msg::Received(Err(s)) => {
				self.server_data.push_str(&format!(
					"Error when reading data from server: {}\n",
					&s.to_string()
				));
				true
			}
			Msg::SendReq => {
				let request = Request::get("/cookies")
					.header("username", "woemaster")
					.header("game_id", "1111")
					.header("viewtype", "player")
					.body(Nothing)
					.unwrap();

				
				let task = fetch::FetchService::fetch(request, self.link.callback(|response: Response<Result<String, Error>>| {
					if response.status().is_success() {
						// response.
						Msg::Received(Ok("HIIIIIII".to_string()))
					} else {
						Msg::Ignore
					}
				})).unwrap();

				ConsoleService::log("hi");
				false
			},
		}
	}
	fn view(&self) -> Html {
		let onbuttonconnect = self.link.callback(|_| Msg::Connect);
		let onbuttonsend = self.link.callback(|_| Msg::SendText);
		let inputtext = self.link.callback(|e: InputData| Msg::TextInput(e.value));
		let sendreq = self.link.callback(|_| Msg::SendReq);
		html! {
			<>
			<input type="text" placeholder="Game ID" style="margin:auto"/>
			<input type="text" placeholder="Username" style="margin:auto"/>
			// connect button
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
			</>
		}
	}
	fn change(&mut self, _: <Self as yew::Component>::Properties) -> bool {
		todo!()
	}
}

// fn main() {
// 	yew::initialize();
// 	App::<Model>::new().mount_to_body();
// 	yew::run_loop();
// }
#[wasm_bindgen(start)]
pub fn run_app() {
	App::<Model>::new().mount_to_body();
}
