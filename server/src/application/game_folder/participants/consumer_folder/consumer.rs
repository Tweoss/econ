use actix::prelude::*;
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;

// use crate::application::other_messages;

// use crate::application::app::AppState;
use crate::application::game_folder::game::Game;

use crate::application::game_folder::game_to_participant;

pub struct ConsumerState {
	pub is_responsive: bool,
	pub took_turn: bool,
	pub score: i64,
	pub addr: Option<Addr<Consumer>>,
}

impl ConsumerState {
	pub fn new() -> ConsumerState {
		ConsumerState {
			is_responsive: true,
			took_turn: false,
			score: 0,
			addr: None,
		}
	}
}

/// Define HTTP actor
pub struct Consumer {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
}

impl Actor for Consumer {
	type Context = ws::WebsocketContext<Self>;
}

impl Consumer {
	// pub async fn new(uuid: String, game_id: String, addr: &actix_web::web::Data<Addr<AppState>>) -> Option<Consumer> {
	// 	if let Some(game_addr) = addr.send(IsConsumer {user_id: uuid.clone(), game_id: game_id.clone()}).await.unwrap() {
	// 		Some(Consumer {uuid, game_id, game_addr})
	// 	}
	// 	else {
	// 		None
	// 	}
	// }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Consumer {
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

impl Handler<game_to_participant::EndedGame> for Consumer {
	type Result = ();
	fn handle(&mut self, _msg: game_to_participant::EndedGame, ctx: &mut Self::Context) -> Self::Result {
		ctx.stop();
	}
}

