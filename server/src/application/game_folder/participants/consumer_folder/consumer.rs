use actix::prelude::*;
use actix::StreamHandler;
use actix_web_actors::ws;
use std::time::Instant;

// use crate::application::app::AppState;
use crate::application::game_folder::game::Game;
use crate::application::game_folder::game_to_participant;
use crate::application::game_folder::game_to_consumer;

use crate::application::game_folder::participants::participant_to_game;
use crate::application::game_folder::participants::consumer_folder::consumer_to_game;
use serde_cbor::{from_slice, to_vec};

use crate::application::game_folder::participants::consumer_folder::consumer_structs::{
	self, ConsumerClientMsg, ConsumerClientType, ConsumerServerMsg, ConsumerServerType,
	ServerExtraFields,
};

use crate::application::game_folder::participants::json::{
	CLIENT_TERMINATE, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL,
};

const INITIAL_BALANCE: f64 = 4000.;
const CLIENT_T_CALCULATION_FREEDOM: f64 = 0.0001;

pub struct ConsumerState {
	pub is_responsive: bool,
	pub took_turn: bool,
	pub score: f64,
	pub balance: f64,
	pub addr: Option<Addr<Consumer>>,
	pub name: String,
	pub quantity_purchased: f64,
}

impl ConsumerState {
	pub fn new(name: String) -> ConsumerState {
		ConsumerState {
			is_responsive: true,
			took_turn: false,
			score: 0.,
			addr: None,
			name,
			balance: INITIAL_BALANCE,
			quantity_purchased: 0.,
		}
	}
}

/// Define HTTP actor
pub struct Consumer {
	uuid: String,
	game_id: String,
	game_addr: Addr<Game>,
	is_producer_turn: bool,
	took_turn: bool,
	trending: u8,
	balance: f64,
	quantity_purchased: f64,
	score: f64,
	hb: Instant,
	is_unresponsive: bool,
}

impl Actor for Consumer {
	type Context = ws::WebsocketContext<Self>;
	fn started(&mut self, ctx: &mut Self::Context) {
		self.game_addr
			.do_send(consumer_to_game::RegisterAddressGetInfo {
				user_id: self.uuid.clone(),
				addr: ctx.address(),
			});
		self.hb(ctx);
	}
	fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
		println!(
			"Stopping a consumer actor: {} and {}",
			self.game_id, self.uuid
		);
		self.game_addr.do_send(participant_to_game::Disconnected {
			id: self.uuid.clone(),
			participant_type: "consumer".to_owned(),
		});
		ctx.terminate();
		Running::Stop
	}
}

impl Consumer {
	pub fn new(uuid: String, game_id: String, game_addr: Addr<Game>) -> Consumer {
		Consumer {
			uuid,
			game_id,
			game_addr,
			trending: 0,
			balance: 0.,
			took_turn: false,
			quantity_purchased: 0.,
			score: 0.,
			hb: Instant::now(),
			is_unresponsive: false,
			is_producer_turn: false,
		}
	}
	fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
		ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
			if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
				act.game_addr.do_send(participant_to_game::Unresponsive {
					id: act.uuid.clone(),
					participant_type: "consumer".to_owned(),
				});
				if Instant::now().duration_since(act.hb) > CLIENT_TERMINATE {
					ctx.binary(
						to_vec(&ConsumerServerMsg {
							msg_type: ConsumerServerType::ServerKicked,
							extra_fields: None,
						})
						.unwrap(),
					);
					act.game_addr.do_send(participant_to_game::Disconnected {
						id: act.uuid.clone(),
						participant_type: "consumer".to_owned(),
					});
					ctx.stop();
				}
				act.is_unresponsive = true;
			}
			let ping = to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::Ping,
				extra_fields: None,
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
				participant_type: "producer".to_string(),
			});
			self.is_unresponsive = false;
		}
	}
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Consumer {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		if let Ok(ws::Message::Binary(bin)) = msg {
			if let Ok(message) = from_slice::<ConsumerClientMsg>(&bin.to_vec()) {
				println!("{:?}", message);
				match message.msg_type {
					ConsumerClientType::Choice => {
						if !self.took_turn && !self.is_producer_turn {
							let choice = message.choice.unwrap();
							// match self.try_produce(choice.quantity, choice.t, choice.price) {
							// 	Ok(score) => {
							// 		let extra_fields = Some(ServerExtraFields {
							// 			submitted_info: Some((score, self.balance)),
							// 			..Default::default()
							// 		});
							// 		ctx.binary(
							// 			to_vec(&ProducerServerMsg {
							// 				msg_type: ProducerServerType::ChoiceSubmitted,
							// 				extra_fields,
							// 			})
							// 			.unwrap(),
							// 		);
							// 		self.took_turn = true;
							// 	}
							// 	Err(msg) => {
							// 		let extra_fields = Some(ServerExtraFields {
							// 			fail_info: Some(msg),
							// 			..Default::default()
							// 		});
							// 		ctx.binary(
							// 			to_vec(&ProducerServerMsg {
							// 				msg_type: ProducerServerType::ChoiceFailed,
							// 				extra_fields,
							// 			})
							// 			.unwrap(),
							// 		);
							// 	}
							// }
						}
					}
					ConsumerClientType::Pong => (),
					// _ => (),
				}
			} else {
				println!("Invalid structure received");
			}
		}
		self.reset_hb();
	}
}

impl Handler<game_to_participant::EndedGame> for Consumer {
	type Result = ();
	fn handle(&mut self, _msg: game_to_participant::EndedGame, ctx: &mut Self::Context) -> Self::Result {
		ctx.stop();
	}
}

impl Handler<game_to_participant::NewOffsets> for Consumer {
	type Result = ();
	fn handle(&mut self, msg: game_to_participant::NewOffsets, ctx: &mut Self::Context) -> Self::Result {
		let fields = ServerExtraFields {
			offsets: Some(consumer_structs::Offsets {
				trending: msg.trending,
			}),
			..Default::default()
		};
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::NewOffsets,
				extra_fields: Some(fields),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::TurnAdvanced> for Consumer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::TurnAdvanced,
		ctx: &mut Self::Context,
	) -> Self::Result {
		self.took_turn = false;
		self.is_producer_turn = !self.is_producer_turn;
		if !self.is_producer_turn {
			self.score += self.balance;
			self.balance = INITIAL_BALANCE;
			let fields = ServerExtraFields {
				balance_score: Some((INITIAL_BALANCE, self.score)),
				..Default::default()
			};
			ctx.binary(
				to_vec(&ConsumerServerMsg {
					msg_type: ConsumerServerType::TurnAdvanced,
					extra_fields: Some(fields),
				})
				.unwrap(),
			);
		} else {
			ctx.binary(
				to_vec(&ConsumerServerMsg {
					msg_type: ConsumerServerType::TurnAdvanced,
					extra_fields: None,
				})
				.unwrap(),
			);
		}
	}
}

impl Handler<game_to_consumer::Info> for Consumer {
	type Result = ();
	fn handle(&mut self, msg: game_to_consumer::Info, ctx: &mut Self::Context) -> Self::Result {
		self.trending = msg.info.trending;
		self.balance = msg.info.balance;
		self.score = msg.info.score;
		self.took_turn = msg.info.took_turn;
		self.is_producer_turn = msg.info.turn % 2 == 1;
		let extra_fields = consumer_structs::ServerExtraFields {
			info: Some(msg.info),
			..Default::default()
		};
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::Info,
				extra_fields: Some(extra_fields),
			})
			.unwrap(),
		);
	}
}