use actix::Running;
use actix::{Actor, ActorContext, Addr, AsyncContext};
use std::time::Duration;
use std::time::Instant;
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;

use actix::prelude::*;

use crate::application::app::AppState;
use crate::application::game_folder::game::Game;

use crate::application::game_folder::participants::director_folder::director_to_game;
use crate::application::game_folder::participants::participant_to_game;
use crate::application::game_folder::participants::json::{DirectorClientMsg, DirectorClientType, DirectorServerMsg, DirectorServerType};

use crate::application::game_folder::game_to_director;
use crate::application::game_folder::game_to_participant;

use serde_cbor::{from_slice, to_vec};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Define HTTP actor
pub struct Director {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
	// pub app_addr: actix_web::web::Data<Addr<AppState>>,
	hb: Instant,
}

impl Actor for Director {
	type Context = ws::WebsocketContext<Self>;
	//* giving the game the address
	fn started(&mut self, ctx: &mut Self::Context) {
		// self.game_addr.send(ws_to_game::ConnectingDirector {user_id: self.uuid})
		self.game_addr.do_send(director_to_game::RegisterAddress {user_id: self.uuid.clone(), addr: ctx.address()});
		self.hb(ctx);
	}
	fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
		println!(
			"Stopping a director actor: {} and {}",
			self.game_id, self.uuid
		);
		Running::Stop
	}
}

impl Director {
	pub async fn new(
		uuid: String,
		game_id: String,
		game_addr: Addr<Game>,
		// addr: actix_web::web::Data<Addr<AppState>>,
	) -> Director {
		Director {
			uuid,
			game_id,
			game_addr,
			// app_addr: addr,
			hb: Instant::now(),
		}
	}
	fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
		ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
			println!("Sending Heartbeat");
			// check client heartbeats
			if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
				// heartbeat timed out
				println!("Websocket Client heartbeat failed!");

				// notify chat server
				act.game_addr.do_send(participant_to_game::Unresponsive { id: act.uuid.clone() });

				// stop actor
				// ctx.stop();

				// don't try to send a ping
				// return;
			}

			ctx.ping(b"");
		});
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
				let message: DirectorClientMsg = from_slice::<DirectorClientMsg>(&bin.to_vec()).unwrap();
				println!("{:?}", message);
				match message.msg_type {
					DirectorClientType::EndGame => {
						println!("Sup");
						self.game_addr.do_send(director_to_game::EndGame {});
						// self.app_addr.do_send(director_to_app::EndGame {
						// 	game_id: self.game_id.clone(),
						// });
						// ctx.close(Some(actix_web_actors::ws::CloseReason::from(
						// 	actix_web_actors::ws::CloseCode::Normal,
						// )));
						// ctx.close(None);
						// ctx.stop();
					}
					DirectorClientType::OpenGame => {}
					_ => (),
				}
				let response = to_vec(&DirectorServerMsg {msg_type: DirectorServerType::Ignore, target: None}).unwrap();
				ctx.binary(response);
			}
			_ => (),
		}
	}
}

impl Handler<game_to_director::Unresponsive> for Director {
	type Result = ();
	fn handle(&mut self, msg: game_to_director::Unresponsive, _: &mut Self::Context) -> Self::Result {
	}
}

impl Handler<game_to_participant::EndedGame> for Director {
	type Result = ();
	fn handle(&mut self, _msg: game_to_participant::EndedGame, ctx: &mut Self::Context) -> Self::Result {
		println!("SUP");
		ctx.stop()
	}
}