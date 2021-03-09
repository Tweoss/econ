#![recursion_limit = "1024"]
use wasm_bindgen::prelude::*;

// #[macro_use]

// #[macro_use]
extern crate failure;
extern crate yew;

// use failure::Error;
use anyhow::Error;

use http::{Request, Response};
use stdweb::js;
use yew::format::Json;
use yew::html::ComponentLink;
use yew::prelude::*;
use yew::services::fetch;
use yew::services::ConsoleService;

use serde_json::json;

struct Model {
	// console: ConsoleService,
	// ws: Option<WebSocketTask>,
	link: ComponentLink<Model>,
	game_id: String, // text in our input box
	name: String,
	password: String,
	server_data: String, // data received from the server
	task: Option<fetch::FetchTask>,
}

enum Msg {
	Ignore,              // ignore this message
	GameIDInput(String), // text was input in the input box
	NameInput(String),   // text was input in the input box
	SendReq,
	Received(String),
	PasswordInput(String),
}

impl Component for Model {
	type Message = Msg;
	type Properties = ();

	fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
		Model {
			// console: ConsoleService {},
			// ws: None,
			link,
			game_id: String::new(),
			name: String::new(),
			password: String::new(),
			server_data: String::new(),
			task: None,
		}
	}

	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		match msg {
			Msg::Ignore => false,
			Msg::GameIDInput(e) => {
				self.game_id = e; // note input box value
				false
			}
			Msg::NameInput(e) => {
				self.name = e; // note input box value
				false
			}
			Msg::SendReq => {
				ConsoleService::log(&self.password);
				let json = json!({"username": self.name, "viewtype": "director", "game_id": self.game_id, "password": &self.password});
				let post_request = Request::post("/cookies")
					// let post_request = Request::post("https://a.valour.vision/cookies")
					// .header("Content-Type", "text/plain")
					.header("Content-Type", "application/json")
					.body(Json(&json))
					.unwrap();
				// .expect("Could not build that request.");
				// let options = FetchOptions {
				// 	credentials: Some(web_sys::RequestCredentials::SameOrigin),
				// 	..FetchOptions::default()
				// };
				let callback = self
					.link
					.callback(|response: Response<Result<String, Error>>| {
						if response.status().is_success() {
							// response.
							ConsoleService::log("Sent Request and Received Response with code: ");
							ConsoleService::log(response.status().as_str());
							if response.body().as_ref().unwrap() == "Success" {
								js! {
									document.getElementById("link").click();
								}
							}
							Msg::Received(response.body().as_ref().unwrap().to_string())
						} else {
							ConsoleService::log("Failed to Send Request");
							Msg::Received(format!("Failed to send request: {}", response.status()))
						}
					});
				let task = fetch::FetchService::fetch(
					post_request,
					// options,
					callback,
				)
				.unwrap();
				self.task = Some(task);
				false
			}
			Msg::Received(data) => {
				self.server_data = data;
				true
			}
			Msg::PasswordInput(e) => {
				self.password = e;
				false
			}
		}
	}
	fn view(&self) -> Html {
		let input_gameid = self.link.callback(|e: InputData| Msg::GameIDInput(e.value));
		let input_name = self.link.callback(|e: InputData| Msg::NameInput(e.value));
		let input_password = self
			.link
			.callback(|e: InputData| Msg::PasswordInput(e.value));
		let sendreq = self.link.callback(|_| Msg::SendReq);
		html! {
			<>
			<div class="container">
				<input id="game_id" type="text" oninput=input_gameid placeholder="Game ID" class="input" maxlength="6" autocomplete="off"/>
				<br/>
				<input id="name" oninput=input_name placeholder="Username" class="input" maxlength="20" autocomplete="off"/>
				<br/>
				<input id="password" oninput=input_password placeholder="Password" class="input" maxlength="12" autocomplete="off"/>
				<br/>
				<button id="enter" class="enter" onclick=sendreq>{"Enter"}</button>
				<p class="message">{self.server_data.clone()}</p>
			</div>
			<a id="link" class="link" href="/redirect"></a>
			// // connect button
			// <p><button onclick=onbuttonconnect,>{ "Connect" }</button></p><br/>
			// // text showing whether we're connected or not
			// <p>{ "Connected: " } { !self.ws.is_none() } </p><br/>
			// // input box for sending text
			// <p><input type="text", value=&self.text, oninput=inputtext,/></p><br/>
			// // button for sending text
			// <p><button onclick=onbuttonsend,>{ "Send" }</button></p><br/>
			// // text area for showing data from the server
			// <p><textarea value=&self.server_data,></textarea></p><br/>
			// // button to send request
			// <p><button onclick=sendreq,>{ "Send Req" }</button></p><br/>
			</>
		}
	}
	fn change(&mut self, _: <Self as yew::Component>::Properties) -> bool {
		todo!()
	}
	fn rendered(&mut self, first_render: bool) {
		if first_render {
			js! {
				document.getElementById("password").addEventListener("keyup", function(event) {
					// Number 13 is the "Enter" key on the keyboard
					if (event.keyCode === 13) {
					  // Cancel the default action, if needed
					  event.preventDefault();
					  // Trigger the button element with a click
					  document.getElementById("enter").click();
					}
				  });
			}
		}
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
