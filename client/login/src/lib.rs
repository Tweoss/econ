#![recursion_limit = "1024"]
use wasm_bindgen::prelude::*;

extern crate failure;
extern crate yew;

use anyhow::Error;
extern crate console_error_panic_hook;
use std::panic;

use http::{Request, Response};
use stdweb::js;
use yew::format::Json;
use yew::html::ComponentLink;
use yew::prelude::*;
use yew::services::fetch;
use yew::services::ConsoleService;

use serde_json::json;

struct Model {
	link: ComponentLink<Model>,
	game_id: String, // text in our input box
	name: String,
	server_data: String, // data received from the server
	task: Option<fetch::FetchTask>,
}

enum Msg {
	GameIdInput(String), // text was input in the input box
	NameInput(String),   // text was input in the input box
	SendReq,
	Receieved(String),
}

impl Component for Model {
	type Message = Msg;
	type Properties = ();

	fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
		Model {
			link,
			game_id: String::new(),
			name: String::new(),
			server_data: String::new(),
			task: None,
		}
	}

	fn update(&mut self, msg: Self::Message) -> ShouldRender {
		match msg {
			Msg::GameIdInput(e) => {
				self.game_id = e; // note input box value
				true
			}
			Msg::NameInput(e) => {
				self.name = e; // note input box value
				true
			}
			Msg::SendReq => {
				let json = json!({"username": self.name, "viewtype": "player", "game_id": self.game_id, "password": ""});
				let post_request = Request::post("/cookies")
					.header("Content-Type", "application/json")
					.body(Json(&json))
					.unwrap();
				let callback = self
					.link
					.callback(|response: Response<Result<String, Error>>| {
						if response.status().is_success() {
							ConsoleService::log("Sent Request and Received Response with code: ");
							ConsoleService::log(response.status().as_str());
							if response.body().as_ref().unwrap() == "Success" {
								js! {
									document.getElementById("link").click();
								}
							}
							Msg::Receieved(response.body().as_ref().unwrap().to_string())
						} else {
							ConsoleService::log("Failed to Send Request");
							Msg::Receieved(format!("Failed to send request: {}", response.status()))
						}
					});
				let task = fetch::FetchService::fetch(
					post_request,
					callback,
				)
				.unwrap();
				self.task = Some(task);
				false
			}
			Msg::Receieved(data) => {
				self.server_data = data;
				true
			}
		}
	}
	fn view(&self) -> Html {
		let input_gameid = self.link.callback(|e: InputData| Msg::GameIdInput(e.value));
		let input_name = self.link.callback(|e: InputData| Msg::NameInput(e.value));
		let sendreq = self.link.callback(|_| Msg::SendReq);
		html! {
			<>
			<div class="container">
				<input id="game_id" type="text" oninput=input_gameid placeholder="Game ID" class="input" maxlength="6" autocomplete="off"/>
				<br/>
				<input id="name" oninput=input_name placeholder="Username" class="input" maxlength="20" autocomplete="off"/>
				<br/>
				<button id="enter" class="enter" onclick=sendreq>{"Enter"}</button>
				<p class="message">{self.server_data.clone()}</p>
			</div>
			<a id="link" class="link" href="/redirect"></a>
			</>
		}
	}
	fn change(&mut self, _: <Self as yew::Component>::Properties) -> bool {
		todo!()
	}
	fn rendered(&mut self, first_render: bool) {
		if first_render {
			js! {
				document.getElementById("name").addEventListener("keyup", function(event) {
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

#[wasm_bindgen(start)]
pub fn run_app() {
	panic::set_hook(Box::new(console_error_panic_hook::hook));
	App::<Model>::new().mount_to_body();
}
