use actix::{Addr, Actor, Context};
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;

// use crate::application::other_messages;

use crate::application::app::AppState;
use crate::application::game::Game;

// mod application;
use crate::application::handle_to_app::*;

// //* players can make actions to be sent to the game to manage
pub struct Player {
	//* will never be modified
	pub uuid: String,
	pub username: String,
	//* will never be modified
	// game_addr: Addr<Game>,
}

impl Actor for Player {
	type Context = Context<Self>;
}

impl Player {
	pub fn new(uuid: String, username: String) -> Player {
		Player { uuid, username }
	}
}

/// Define HTTP actor
pub struct Director {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
}

impl Actor for Director {
	type Context = ws::WebsocketContext<Self>;
}

impl Director {
	pub async fn new(uuid: String, game_id: String, addr: &actix_web::web::Data<Addr<AppState>>) -> Option<Director> {
		if let Some(game_addr) = addr.send(IsDirector {user_id: uuid.clone(), game_id: game_id.clone()}).await.unwrap() {
			Some(Director {uuid, game_id, game_addr})
		}
		else {
			None
		}
		// Director {
		// 	uuid,
		// 	game_id,
		// }
	}
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Director {
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
