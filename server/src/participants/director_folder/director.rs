use actix::Running;
use actix::StreamHandler;
use actix::{Actor, ActorContext, Addr, AsyncContext};
use actix_web_actors::ws;
use std::time::Instant;

use actix::prelude::*;

use crate::game_folder::game::Game;

use crate::participants::director_folder::director_structs::{
	self, DirectorClientMsg, DirectorClientType, DirectorServerMsg, DirectorServerType,
};
use crate::participants::director_folder::director_to_game;
use crate::participants::participant_to_game;

use crate::game_folder::game_to_director;
use crate::game_folder::game_to_participant;

use super::director_structs::ParticipantType;

use serde_cbor::{from_slice, to_vec};

use crate::participants::heartbeat::{CLIENT_TERMINATE, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL};

pub struct DirectorState {
	pub is_responsive: bool,
	pub addr: Option<Addr<Director>>,
	pub id: String,
}

impl DirectorState {
	pub fn new(id: String) -> DirectorState {
		DirectorState {
			is_responsive: true,
			addr: None,
			id,
		}
	}
}

/// Define HTTP actor
pub struct Director {
	pub name: String,
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
				name: self.name.clone(),
				addr: ctx.address(),
			});
		self.hb(ctx);
	}
	fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
		println!(
			"Stopping a director actor: {} and {}",
			self.game_id, self.name
		);
		self.game_addr.do_send(participant_to_game::Disconnected {
			name: self.name.clone(),
			participant_type: "director".to_owned(),
		});
		ctx.terminate();
		Running::Stop
	}
}

impl Director {
	pub fn new(name: String, game_id: String, game_addr: Addr<Game>) -> Director {
		Director {
			name,
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
					name: act.name.clone(),
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
						name: act.name.clone(),
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
				name: self.name.clone(),
				participant_type: "director".to_string(),
			});
			self.is_unresponsive = false;
		}
	}
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Director {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
		if let Ok(ws::Message::Binary(bin)) = msg {
			if let Ok(message) = from_slice::<DirectorClientMsg>(&bin.to_vec()) {
				match message.msg_type {
					DirectorClientType::EndGame => {
						self.game_addr.do_send(director_to_game::EndGame {});
					}
					DirectorClientType::OpenGame => {
						self.game_addr.do_send(director_to_game::OpenGame {});
					}
					DirectorClientType::CloseGame => {
						self.game_addr.do_send(director_to_game::CloseGame {});
					}
					DirectorClientType::Pong => {}
					DirectorClientType::Kick(target) => {
						self.game_addr
							.do_send(director_to_game::KickParticipant { name: target });
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
					}
				}
			} else {
				println!("Invalid structure received");
			}
			self.reset_hb();
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
				msg_type: DirectorServerType::UnresponsivePlayer(msg.name, msg.participant_type),
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
				msg_type: DirectorServerType::DisconnectedPlayer(msg.name, msg.participant_type),
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
				msg_type: DirectorServerType::ConnectedPlayer(msg.name, msg.participant_type),
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
				msg_type: DirectorServerType::TurnTaken(msg.name, msg.participant_type),
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
				msg_type: DirectorServerType::ParticipantKicked(msg.name),
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
