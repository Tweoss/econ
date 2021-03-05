use actix::Running;
use actix::{Actor, ActorContext, Addr, AsyncContext};
use std::time::Duration;
use std::time::Instant;
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;

use actix::prelude::*;

// use crate::application::app::AppState;
use crate::application::game_folder::game::Game;

use crate::application::game_folder::participants::director_folder::director_structs::{
	self, DirectorClientMsg, DirectorClientType, DirectorServerMsg, DirectorServerType, ServerExtraField,
};
use crate::application::game_folder::participants::director_folder::director_to_game;
use crate::application::game_folder::participants::participant_to_game;

use crate::application::game_folder::game_to_director;
use crate::application::game_folder::game_to_participant;

use super::director_structs::ParticipantType;

use serde_cbor::{from_slice, to_vec};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct DirectorState {
	pub is_responsive: bool,
	pub addr: Option<Addr<Director>>,
}

impl DirectorState {
	pub fn new() -> DirectorState {
		DirectorState {
			is_responsive: true,
			addr: None,
		}
	}
}

/// Define HTTP actor
pub struct Director {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
	hb: Instant,
}

impl Actor for Director {
	type Context = ws::WebsocketContext<Self>;
	//* giving the game the address
	fn started(&mut self, ctx: &mut Self::Context) {
		self.game_addr
			.do_send(director_to_game::RegisterAddressGetInfo {
				user_id: self.uuid.clone(),
				addr: ctx.address(),
			});
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
			// check client heartbeats
			if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
				// heartbeat timed out
				// notify game
				act.game_addr.do_send(participant_to_game::Unresponsive {
					id: act.uuid.clone(),
				});
			}
			let response = to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::Ping,
				extra_fields: None,
				// target: None,
				// info: None,
			})
			.unwrap();
			ctx.binary(response);
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
				let message: DirectorClientMsg =
					from_slice::<DirectorClientMsg>(&bin.to_vec()).unwrap();
				println!("{:?}", message);
				match message.msg_type {
					DirectorClientType::EndGame => {
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
					DirectorClientType::OpenGame => {
						self.game_addr.do_send(director_to_game::OpenGame {});
					}
					DirectorClientType::CloseGame => {
						self.game_addr.do_send(director_to_game::CloseGame {});
					}
					DirectorClientType::Pong => {
						self.hb = Instant::now();
					}
					DirectorClientType::Kick => {
						self.game_addr.do_send(director_to_game::KickParticipant {
							user_id: message.extra_fields.unwrap().target.unwrap(),
						});
					}
					DirectorClientType::NewOffsets => {
						let offsets = message.extra_fields.unwrap().offsets.unwrap();
						self.game_addr.do_send(director_to_game::SetOffsets {
							subsidies: offsets.subsidies,
							supply_shock: offsets.supply_shock,
							trending: offsets.trending,
						})
					}
					_ => (),
				}
				// let response = to_vec(&DirectorServerMsg {
				// 	msg_type: DirectorServerType::Ignore,
				// 	target: None,
				// })
				// .unwrap();
				// ctx.binary(response);
			}
			_ => (),
		}
	}
}

impl Handler<game_to_director::Info> for Director {
	type Result = ();
	fn handle(&mut self, msg: game_to_director::Info, ctx: &mut Self::Context) -> Self::Result {
		let fields = ServerExtraField {
			info: Some(msg.info),
			..Default::default()
		};
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::Info,
				extra_fields: Some(fields),
			})
			.unwrap(),
		)
	}
}

impl Handler<game_to_director::Unresponsive> for Director {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_director::Unresponsive,
		_: &mut Self::Context,
	) -> Self::Result {
	}
}

impl Handler<game_to_director::NewParticipant> for Director {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_director::NewParticipant,
		ctx: &mut Self::Context,
	) -> Self::Result {
		let fields = ServerExtraField {
			target: Some(msg.id),
			..Default::default()
		};
		match msg.participant_type {
			ParticipantType::Director => ctx.binary(
				to_vec(&DirectorServerMsg {
					msg_type: DirectorServerType::NewDirector,
					extra_fields: Some(fields),
				})
				.unwrap(),
			),
			ParticipantType::Producer => ctx.binary(
				to_vec(&DirectorServerMsg {
					msg_type: DirectorServerType::NewProducer,
					extra_fields: Some(fields),
				})
				.unwrap(),
			),
			ParticipantType::Consumer => ctx.binary(
				to_vec(&DirectorServerMsg {
					msg_type: DirectorServerType::NewConsumer,
					extra_fields: Some(fields),
				})
				.unwrap(),
			),
			ParticipantType::Viewer => ctx.binary(
				to_vec(&DirectorServerMsg {
					msg_type: DirectorServerType::NewViewer,
					extra_fields: Some(fields),
				})
				.unwrap(),
			),
		}
	}
}

impl Handler<game_to_director::KickedParticipant> for Director {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_director::KickedParticipant,
		ctx: &mut Self::Context,
	) -> Self::Result {
		let fields = ServerExtraField {
			target: Some(msg.id),
			..Default::default()
		};
		// fields.target = Some(msg.id);
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::ParticipantKicked,
				extra_fields: Some(fields),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_director::GameOpened> for Director {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_director::GameOpened,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::GameOpened,
				extra_fields: None,
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_director::GameClosed> for Director {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_director::GameClosed,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::GameClosed,
				extra_fields: None,
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::EndedGame> for Director {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::EndedGame,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.stop()
	}
}

impl Handler<game_to_participant::NewOffsets> for Director {
	type Result = ();
	fn handle(&mut self, msg: game_to_participant::NewOffsets, ctx: &mut Self::Context) {
		let fields = ServerExtraField {
			offsets: Some(director_structs::Offsets {
				subsidies: msg.subsidies,
				supply_shock: msg.supply_shock,
				trending: msg.trending,
			}),
			..Default::default()
		};
		// fields.target = Some(msg.id);
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::NewOffsets,
				extra_fields: Some(fields),
			})
			.unwrap(),
		);
	}
}
