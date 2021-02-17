use actix::{Addr, Actor, Context};
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;

// use crate::application::other_messages;

use crate::application::app::AppState;
use crate::application::game::Game;


use crate::application::participants::ws_to_app;
use crate::application::participants::json::{DirectorJson, DirectorMsgType};

use serde_cbor::{from_slice, to_vec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    choice: u64,
    string: String,
}

/// Define HTTP actor
pub struct Director {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
}

impl Actor for Director {
	type Context = ws::WebsocketContext<Self>;
	//* giving the game the address
	fn started(&mut self,_ctx: &mut Self::Context) {
		// self.game_addr.send(ws_to_game::ConnectingDirector {user_id: self.uuid})
	}
	//* called by the game
	fn stopped(&mut self, ctx: &mut Self::Context) {
		// SEND A MESSAGE TO CLIENT TO TERMINATE CONNECTION
	}
}

impl Director {
	pub async fn new(uuid: String, game_id: String, addr: actix_web::web::Data<Addr<AppState>>) -> Option<Director> {
	// pub async fn new(uuid: String, game_id: String, game_addr: Addr<Game>) -> Option<Director> {
		println!("Attempting to make a new Director ws");
		if let Some(game_addr) = addr.send(ws_to_app::IsRegisteredDirector {user_id: uuid.clone(), game_id: game_id.clone()}).await.unwrap() {
			Some(Director {uuid, game_id, game_addr})
		}
		else {
			None
		}
	}
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Director {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		match msg {
			// Ok(ws::Message::Text) => (),
			Ok(ws::Message::Text(text)) => {
				println!("HEY?");
				ctx.text(text);
			}
			Ok(ws::Message::Binary(bin)) => {
				println!("Hey?");
				println!("{:?}", from_slice::<Data>(&bin.to_vec()));
				ctx.binary(bin);
			}
			_ => (),
		}
	}
}
