use actix::prelude::*;
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;

// use crate::application::other_messages;

use crate::application::game_folder::game::Game;
use crate::application::game_folder::game_to_participant;

pub struct ProducerState {
	pub is_connected: bool,
	pub score: i64,
	pub quantity_remaining: u64,
	pub price: i64,
	pub addr: Option<Addr<Producer>>,
}

impl ProducerState {
	pub fn new() -> ProducerState {
		ProducerState {
			is_connected: false,
			score: 0,
			quantity_remaining: 0,
			price: 0,
			addr: None,
		}
	}
}

/// Define HTTP actor
pub struct Producer {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
}

impl Actor for Producer {
	type Context = ws::WebsocketContext<Self>;
}

impl Producer {
	// pub async fn new(uuid: String, game_id: String, addr: &actix_web::web::Data<Addr<AppState>>) -> Option<Producer> {
	// 	if let Some(game_addr) = addr.send(IsProducer {user_id: uuid.clone(), game_id: game_id.clone()}).await.unwrap() {
	// 		Some(Producer {uuid, game_id, game_addr})
	// 	}
	// 	else {
	// 		None
	// 	}
	// }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Producer {
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

impl Handler<game_to_participant::EndedGame> for Producer {
	type Result = ();
	fn handle(&mut self, _msg: game_to_participant::EndedGame, ctx: &mut Self::Context) -> Self::Result {
		ctx.stop();
	}
}