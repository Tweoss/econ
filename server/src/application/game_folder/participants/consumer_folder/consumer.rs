use actix::prelude::*;
use actix::StreamHandler;
use actix_web_actors::ws;
use std::time::Instant;

// use crate::application::app::AppState;
use crate::application::game_folder::game::Game;
use crate::application::game_folder::game_to_consumer;
use crate::application::game_folder::game_to_participant;

use crate::application::game_folder::participants::consumer_folder::consumer_to_game;
use crate::application::game_folder::participants::participant_to_game;
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
	total_utility: f64,
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
			total_utility: 0.,
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
	#[allow(clippy::many_single_char_names)]
	// * determine how much utility would be gained by purchasing
	fn get_utility(&mut self, quantity: f64) -> f64 {
		// * doing t_2 = 2. just in case quantity is out of bounds
		let t = self.get_t_for_quantity(0., 2., quantity + self.quantity_purchased, 50);
		let p = f64::from(self.trending);
		let (a, b, c, d) = (0., 40., 65., 80.);
		let (e, f, g, h) = (80., 80., 70., 0.);

		let new_total = -3. * (a - b) * (e + p) * t
			+ (3. / 2.)
				* (5. * a * e - 7. * b * e + 2. * c * e - 3. * a * f
					+ 3. * b * f + 2. * (a - 2. * b + c) * p)
				* f64::powi(t, 2)
			- (9. * c * e - d * e - 6. * c * f + 3. * c * p
				- d * p - 3. * b * (6. * e - 6. * f + g + p)
				+ a * (10. * e - 12. * f + 3. * g + p))
				* f64::powi(t, 3)
			+ (3. / 4.)
				* (3. * (5. * c * e - d * e - 7. * c * f + d * f + 2. * c * g)
					+ a * (10. * e - 18. * f + 9. * g - h)
					+ b * (-22. * e + 36. * f - 15. * g + h))
				* f64::powi(t, 4)
			- (3. / 5.)
				* (5. * a * e - 13. * b * e + 11. * c * e - 3. * d * e - 12. * a * f + 30. * b * f
					- 24. * c * f + 6. * d * f
					+ 9. * a * g - 21. * b * g
					+ 15. * c * g - 3. * d * g
					- 2. * (a - 2. * b + c) * h)
				* f64::powi(t, 5)
			+ (1. / 2.) * (a - 3. * b + 3. * c - d) * (e - 3. * f + 3. * g - h) * f64::powi(t, 6);
		new_total - self.total_utility
	}
	fn get_t_for_quantity(&self, t_0: f64, t_2: f64, x: f64, iterations: u32) -> f64 {
		if iterations == 0 {
			return t_0;
		}
		let t_1 = (t_0 + t_2) / 2.;
		let x_1 = 3. * f64::powi(1. - t_1, 2) * t_1 * 40.
			+ 3. * (1. - t_1) * f64::powi(t_1, 2) * 65.
			+ f64::powi(t_1, 3) * 80.
			- x;
		if x_1 > 0. {
			self.get_t_for_quantity(t_0, t_1, x, iterations - 1)
		} else if x_1 < 0. {
			self.get_t_for_quantity(t_1, t_2, x, iterations - 1)
		} else {
			t_1
		}
	}
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Consumer {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
		if let Ok(ws::Message::Binary(bin)) = msg {
			if let Ok(message) = from_slice::<ConsumerClientMsg>(&bin.to_vec()) {
				println!("{:?}", message);
				match message.msg_type {
					ConsumerClientType::Choice => {
						if !self.took_turn && !self.is_producer_turn {
							// * send the message to game. game calculates purchased quantity and returns the expense, remaining balance, and total purchased quantity
							self.game_addr.do_send(consumer_to_game::TryChoice {
								user_id: self.uuid.clone(),
								elements: message.choice.unwrap().elements,
							});
						}
					}
					ConsumerClientType::Pong => (),
					ConsumerClientType::EndTurn => {
						self.score += self.balance;
						self.balance = 0.;
						self.game_addr.do_send(consumer_to_game::NewScoreEndTurn {
							user_id: self.uuid.clone(),
							new_score: self.score,
						});
					}
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
	fn handle(
		&mut self,
		_msg: game_to_participant::EndedGame,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.stop();
	}
}

impl Handler<game_to_participant::NewOffsets> for Consumer {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_participant::NewOffsets,
		ctx: &mut Self::Context,
	) -> Self::Result {
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
			self.total_utility = 0.;
			let fields = ServerExtraFields {
				balance_score_quantity: Some((INITIAL_BALANCE, self.score, 0.)),
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
		// let extra_fields = consumer_structs::ServerExtraFields {
		// 	info: Some(msg.info),
		// 	..Default::default()
		// };
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::Info(msg.info),
				extra_fields: None,
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_consumer::PurchaseResult> for Consumer {
	type Result = ();
	fn handle(&mut self, msg: game_to_consumer::PurchaseResult, ctx: &mut Self::Context) {
		if msg.purchased == 0. {
			println!("Attempted to negative purchase");
		}
		else {
			self.balance = msg.balance;
			let utility = self.get_utility(msg.purchased);
			self.quantity_purchased += msg.purchased;
			self.score += utility;
			self.total_utility += utility;
			println!("Consumer says utility: {}, total_utility: {}, score: {}", utility, self.total_utility, self.score);
			self.game_addr.do_send(consumer_to_game::NewScoreCalculated {user_id: self.uuid.clone(), new_score: self.score});
			let fields = ServerExtraFields {
				balance_score_quantity: Some((self.balance, self.score, self.quantity_purchased)),
				..Default::default()
			};
			ctx.binary(
				to_vec(&ConsumerServerMsg {
					msg_type: ConsumerServerType::ChoiceSubmitted,
					extra_fields: Some(fields),
				})
				.unwrap(),
			);
		}
	}
}

impl Handler<game_to_consumer::TurnList> for Consumer {
	type Result = ();
	fn handle(&mut self, msg: game_to_consumer::TurnList, ctx: &mut Self::Context) -> Self::Result {
		let fields = ServerExtraFields {
			turn_info: Some(consumer_structs::TurnInfo {
				producers: msg.list,
			}),
			..Default::default()
		};
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::TurnInfo,
				extra_fields: Some(fields),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::StockReduced> for Consumer {
	type Result = ();
	fn handle(&mut self, msg: game_to_participant::StockReduced, ctx: &mut Self::Context) {
		let fields = ServerExtraFields {
			stock_targets: Some(msg.targets),
			..Default::default()
		};
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::StockReduced,
				extra_fields: Some(fields),
			})
			.unwrap(),
		);
	}
}