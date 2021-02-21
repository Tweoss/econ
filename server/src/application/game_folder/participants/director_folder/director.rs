use actix::Running;
use actix::{Actor, Addr, Context, ActorContext};
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;

// use crate::application::other_messages;

use crate::application::app::AppState;
use crate::application::game_folder::game::Game;

use crate::application::game_folder::participants::director_folder::{director_to_app, director_to_game};
use crate::application::game_folder::participants::json::{DirectorData, DirectorMsgType};

use serde_cbor::{from_slice, to_vec};

/// Define HTTP actor
pub struct Director {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
	pub app_addr: actix_web::web::Data<Addr<AppState>>,
}

impl Actor for Director {
	type Context = ws::WebsocketContext<Self>;
	//* giving the game the address
	fn started(&mut self, _ctx: &mut Self::Context) {
		// self.game_addr.send(ws_to_game::ConnectingDirector {user_id: self.uuid})
	}
	fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
		println!(
			"Stopping a director actor: {} and {}",
			self.game_id, self.uuid
		);
		Running::Stop
	}
	// //* called by the game
	// fn stopped(&mut self, ctx: &mut Self::Context) {
	// 	// SEND A MESSAGE TO CLIENT TO TERMINATE CONNECTION
	// }
}

impl Director {
	// pub async fn new(
	// 	uuid: String,
	// 	game_id: String,
	// 	addr: actix_web::web::Data<Addr<AppState>>,
	// ) -> Option<Director> {
	pub async fn new(uuid: String, game_id: String, game_addr: Addr<Game>, addr: actix_web::web::Data<Addr<AppState>>) -> Director {
		// println!("Attempting to make a new Director ws");
		// if let Some(game_addr) = addr
		// 	.send(director_to_app::IsRegisteredDirector {
		// 		user_id: uuid.clone(),
		// 		game_id: game_id.clone(),
		// 	})
		// 	.await
		// 	.unwrap()
		// {
		// 	Some(Director {
		// 		uuid,
		// 		game_id,
		// 		game_addr,
		// 		app_addr: addr,
		// 	})
		// } else {
		// 	None
		// }
		Director {
			uuid,
			game_id,
			game_addr,
			app_addr: addr
		}
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
			Ok(ws::Message::Binary(bin)) => {
				let message: DirectorData = from_slice::<DirectorData>(&bin.to_vec()).unwrap();
				println!("{:?}", message);
				match message.msg_type {
					DirectorMsgType::CloseGame => {
						println!("Sup");
						self.game_addr.do_send(director_to_game::CloseGame {});
						// self.app_addr.do_send(director_to_app::CloseGame {
						// 	game_id: self.game_id.clone(),
						// });
						// ctx.close(Some(actix_web_actors::ws::CloseReason::from(
						// 	actix_web_actors::ws::CloseCode::Normal,
						// )));
						// ctx.close(None);
						ctx.stop();
					}
					DirectorMsgType::OpenGame => {}
					_ => (),
				}
				ctx.binary(bin);
			}
			_ => (),
		}
	}
}
