use actix::{Addr, Actor};
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;

// use crate::application::other_messages;

use crate::application::game_folder::game::Game;

pub struct ViewerState {
	pub is_connected: bool,
	pub addr: Option<Addr<Viewer>>,
}

impl ViewerState {
	pub fn new() -> ViewerState {
		ViewerState {
			is_connected: false,
			addr: None,
		}
	}
}

/// Define HTTP actor
pub struct Viewer {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
}

impl Actor for Viewer {
	type Context = ws::WebsocketContext<Self>;
}

impl Viewer {
	// pub async fn new(uuid: String, game_id: String, addr: &actix_web::web::Data<Addr<AppState>>) -> Option<Viewer> {
	// 	if let Some(game_addr) = addr.send(IsViewer {user_id: uuid.clone(), game_id: game_id.clone()}).await.unwrap() {
	// 		Some(Viewer {uuid, game_id, game_addr})
	// 	}
	// 	else {
	// 		None
	// 	}
	// }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Viewer {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		match msg {
			// Ok(ws::Message::Text) => (),
			Ok(ws::Message::Text(text)) => {
				ctx.text(text);
			}
			Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
			_ => (),
		}
	}
}
