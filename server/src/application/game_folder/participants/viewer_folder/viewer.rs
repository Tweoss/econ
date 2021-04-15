use actix::prelude::*;
use actix::StreamHandler;
use actix_web_actors::ws;
use std::time::Instant;

// use crate::application::app::AppState;
use crate::application::game_folder::game::Game;
use crate::application::game_folder::game_to_participant;
use crate::application::game_folder::game_to_viewer;

use crate::application::game_folder::participants::participant_to_game;
use crate::application::game_folder::participants::viewer_folder::viewer_to_game;
use serde_cbor::{from_slice, to_vec};

use crate::application::game_folder::participants::viewer_folder::viewer_structs::{
	self, ViewerClientMsg, ViewerClientType, ViewerServerMsg, ViewerServerType,
};

use crate::application::game_folder::participants::heartbeat::{
	CLIENT_TERMINATE, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL,
};

pub struct ViewerState {
	pub is_responsive: bool,
	pub addr: Option<Addr<Viewer>>,
	pub name: String,
}

impl ViewerState {
	pub fn new(name: String) -> ViewerState {
		ViewerState {
			is_responsive: true,
			addr: None,
			name,
		}
	}
}

/// Define HTTP actor
pub struct Viewer {
	uuid: String,
	game_addr: Addr<Game>,
	game_id: String,
	hb: Instant,
	is_unresponsive: bool,
}

impl Actor for Viewer {
	type Context = ws::WebsocketContext<Self>;
	fn started(&mut self, ctx: &mut Self::Context) {
		self.game_addr
			.do_send(viewer_to_game::RegisterAddressGetInfo {
				user_id: self.uuid.clone(),
				addr: ctx.address(),
			});
		self.hb(ctx);
	}
	fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
		println!(
			"Stopping a viewer actor: {} and {}",
			self.game_id, self.uuid
		);
		self.game_addr.do_send(participant_to_game::Disconnected {
			id: self.uuid.clone(),
			participant_type: "viewer".to_owned(),
		});
		ctx.terminate();
		Running::Stop
	}
}

impl Viewer {
	pub fn new(uuid: String, game_id: String, game_addr: Addr<Game>) -> Viewer {
		Viewer {
			uuid,
			game_addr,
			game_id,
			hb: Instant::now(),
			is_unresponsive: false,
		}
	}
	fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
		ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
			if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
				act.game_addr.do_send(participant_to_game::Unresponsive {
					id: act.uuid.clone(),
					participant_type: "viewer".to_owned(),
				});
				if Instant::now().duration_since(act.hb) > CLIENT_TERMINATE {
					ctx.binary(
						to_vec(&ViewerServerMsg {
							msg_type: ViewerServerType::ServerKicked,
						})
						.unwrap(),
					);
					act.game_addr.do_send(participant_to_game::Disconnected {
						id: act.uuid.clone(),
						participant_type: "viewer".to_owned(),
					});
					ctx.stop();
				}
				act.is_unresponsive = true;
			}
			let ping = to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::Ping,
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
				participant_type: "viewer".to_string(),
			});
			self.is_unresponsive = false;
		}
	}
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Viewer {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
		if let Ok(ws::Message::Binary(bin)) = msg {
			if let Ok(message) = from_slice::<ViewerClientMsg>(&bin.to_vec()) {
				println!("{:?}", message);
				match message.msg_type {
					ViewerClientType::Pong => (),
				}
			} else {
				println!("Invalid structure received");
			}
		}
		self.reset_hb();
	}
}

impl Handler<game_to_participant::EndedGame> for Viewer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::EndedGame,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::GameEnded,
			})
			.unwrap(),
		);
		ctx.stop();
	}
}

impl Handler<game_to_participant::NewOffsets> for Viewer {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_participant::NewOffsets,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::NewOffsets(viewer_structs::Offsets {
					trending: msg.trending,
					subsidies: msg.subsidies,
					supply_shock: msg.supply_shock,
				}),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::TurnAdvanced> for Viewer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::TurnAdvanced,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::TurnAdvanced,
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::Kicked> for Viewer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::Kicked,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::ServerKicked,
			})
			.unwrap(),
		);
		ctx.terminate();
	}
}

impl Handler<game_to_viewer::Info> for Viewer {
	type Result = ();
	fn handle(&mut self, msg: game_to_viewer::Info, ctx: &mut Self::Context) -> Self::Result {
		ctx.binary(
			to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::Info(msg.info),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_viewer::GameOpened> for Viewer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_viewer::GameOpened,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::GameOpened,
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_viewer::GameClosed> for Viewer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_viewer::GameClosed,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::GameClosed,
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_viewer::NewScores> for Viewer {
	type Result = ();
	fn handle(&mut self, msg: game_to_viewer::NewScores, ctx: &mut Self::Context) -> Self::Result {
		ctx.binary(
			to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::NewScores(msg.list),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_viewer::NewParticipant> for Viewer {
	type Result = ();
	fn handle(&mut self, msg: game_to_viewer::NewParticipant, ctx: &mut Self::Context) -> Self::Result {
		ctx.binary(
			to_vec(&ViewerServerMsg {
				msg_type: ViewerServerType::NewParticipant(
					viewer_structs::Participant {
						name: msg.name,
						is_consumer: msg.is_consumer,
						score: 0.,
						next_index: 0,
					}
				),
			})
			.unwrap(),
		);
	}
}
