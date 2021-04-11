use actix::Running;
use actix::{Actor, ActorContext, Addr, AsyncContext};
// use std::time::Duration;
use std::time::Instant;
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;

use actix::prelude::*;

// use crate::application::app::AppState;
use crate::application::game_folder::game::Game;

use crate::application::game_folder::participants::director_folder::director_structs::{
	self, DirectorClientMsg, DirectorClientType, DirectorServerMsg, DirectorServerType,
};
use crate::application::game_folder::participants::director_folder::director_to_game;
use crate::application::game_folder::participants::participant_to_game;

use crate::application::game_folder::game_to_director;
use crate::application::game_folder::game_to_participant;

use super::director_structs::ParticipantType;

use serde_cbor::{from_slice, to_vec};

use crate::application::game_folder::participants::heartbeat::{
	CLIENT_TERMINATE, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL,
};

pub struct DirectorState {
	pub is_responsive: bool,
	pub addr: Option<Addr<Director>>,
	pub name: String,
}

impl DirectorState {
	pub fn new(name: String) -> DirectorState {
		DirectorState {
			is_responsive: true,
			addr: None,
			name,
		}
	}
}

/// Define HTTP actor
pub struct Director {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
	hb: Instant,
	is_unresponsive: bool,
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
	fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
		println!(
			"Stopping a director actor: {} and {}",
			self.game_id, self.uuid
		);
		self.game_addr.do_send(participant_to_game::Disconnected {
			id: self.uuid.clone(),
			participant_type: "director".to_owned(),
		});
		ctx.terminate();
		Running::Stop
	}
}

impl Director {
	pub fn new(
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
			is_unresponsive: false,
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
					participant_type: "director".to_owned(),
				});
				if Instant::now().duration_since(act.hb) > CLIENT_TERMINATE {
					ctx.binary(
						to_vec(&DirectorServerMsg {
							msg_type: DirectorServerType::ServerKicked,
						})
						.unwrap(),
					);
					act.game_addr.do_send(participant_to_game::Disconnected {
						id: act.uuid.clone(),
						participant_type: "director".to_owned(),
					});
					ctx.stop();
				}
				act.is_unresponsive = true;
			}
			let ping = to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::Ping,
			})
			.unwrap();
			ctx.binary(ping);
		});
	}
	fn reset_hb(&mut self) {
		self.hb = Instant::now();
		if self.is_unresponsive {
			self.game_addr.do_send(participant_to_game::Responsive {
				id: self.uuid.clone(),
				participant_type: "director".to_string(),
			});
			self.is_unresponsive = false;
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
				if let Ok(message) = from_slice::<DirectorClientMsg>(&bin.to_vec()) {
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
							// self.hb = Instant::now();
						}
						DirectorClientType::Kick(target) => {
							self.game_addr
								.do_send(director_to_game::KickParticipant { user_id: target });
						}
						DirectorClientType::NewOffsets(offsets) => {
							let offsets = offsets;
							self.game_addr.do_send(director_to_game::SetOffsets {
								subsidies: offsets.subsidies,
								supply_shock: offsets.supply_shock,
								trending: offsets.trending,
							})
						}
						DirectorClientType::NextTurn => {
							self.game_addr.do_send(director_to_game::ForceTurn {});
						} // _ => (),
					}
				} else {
					println!("Invalid structure received");
				}
				self.reset_hb();
				// self.hb = Instant::now();
			}
			_ => (),
		}
	}
}

impl Handler<game_to_director::Info> for Director {
	type Result = ();
	fn handle(&mut self, msg: game_to_director::Info, ctx: &mut Self::Context) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::Info(msg.info),
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
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::UnresponsivePlayer(msg.id, msg.participant_type),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_director::Disconnected> for Director {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_director::Disconnected,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::DisconnectedPlayer(msg.id, msg.participant_type),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_director::Connected> for Director {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_director::Connected,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::ConnectedPlayer(msg.id, msg.participant_type),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_director::TurnTaken> for Director {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_director::TurnTaken,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::TurnTaken(msg.id, msg.participant_type),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_director::NewParticipant> for Director {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_director::NewParticipant,
		ctx: &mut Self::Context,
	) -> Self::Result {
		match msg.participant_type {
			ParticipantType::Director => ctx.binary(
				to_vec(&DirectorServerMsg {
					msg_type: DirectorServerType::NewDirector(msg.id, msg.name),
				})
				.unwrap(),
			),
			ParticipantType::Producer => ctx.binary(
				to_vec(&DirectorServerMsg {
					msg_type: DirectorServerType::NewProducer(msg.id, msg.name),
				})
				.unwrap(),
			),
			ParticipantType::Consumer => ctx.binary(
				to_vec(&DirectorServerMsg {
					msg_type: DirectorServerType::NewConsumer(msg.id, msg.name),
				})
				.unwrap(),
			),
			ParticipantType::Viewer => ctx.binary(
				to_vec(&DirectorServerMsg {
					msg_type: DirectorServerType::NewViewer(msg.id, msg.name),
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
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::ParticipantKicked(msg.id),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_director::GameOpened> for Director {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_director::GameOpened,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::GameOpened,
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_director::GameClosed> for Director {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_director::GameClosed,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::GameClosed,
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
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::GameEnded,
			})
			.unwrap(),
		);
		ctx.stop()
	}
}

impl Handler<game_to_participant::NewOffsets> for Director {
	type Result = ();
	fn handle(&mut self, msg: game_to_participant::NewOffsets, ctx: &mut Self::Context) {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::NewOffsets(director_structs::Offsets {
					subsidies: msg.subsidies,
					supply_shock: msg.supply_shock,
					trending: msg.trending,
				}),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::TurnAdvanced> for Director {
	type Result = ();
	fn handle(&mut self, _: game_to_participant::TurnAdvanced, ctx: &mut Self::Context) {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::TurnAdvanced,
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::Kicked> for Director {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::Kicked,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&DirectorServerMsg {
				msg_type: DirectorServerType::ServerKicked,
			})
			.unwrap(),
		);
		ctx.terminate();
	}
}
