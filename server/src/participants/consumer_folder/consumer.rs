use actix::prelude::*;
use actix::StreamHandler;
use actix_web_actors::ws;
use std::time::Instant;

use crate::game_folder::game::Game;
use crate::game_folder::game_to_consumer;
use crate::game_folder::game_to_participant;

use crate::participants::consumer_folder::consumer_to_game;
use crate::participants::participant_to_game;
use serde_cbor::{from_slice, to_vec};

use crate::participants::consumer_folder::consumer_structs::{
	self, ConsumerClientMsg, ConsumerClientType, ConsumerServerMsg, ConsumerServerType,
};

use crate::participants::heartbeat::{CLIENT_TERMINATE, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL};

const INITIAL_BALANCE: f64 = 4000.;

pub struct ConsumerState {
	pub is_responsive: bool,
	pub took_turn: bool,
	pub score: f64,
	pub balance: f64,
	pub addr: Option<Addr<Consumer>>,
	pub id: String,
	pub quantity_purchased: f64,
}

impl ConsumerState {
	pub fn new(id: String) -> ConsumerState {
		ConsumerState {
			is_responsive: true,
			took_turn: false,
			score: 0.,
			addr: None,
			id,
			balance: INITIAL_BALANCE,
			quantity_purchased: 0.,
		}
	}
}

pub struct Consumer {
	name: String,
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
				name: self.name.clone(),
				addr: ctx.address(),
			});
		self.hb(ctx);
	}
	fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
		println!(
			"Stopping a consumer actor: {} and {}",
			self.game_id, self.name
		);
		self.game_addr.do_send(participant_to_game::Disconnected {
			name: self.name.clone(),
			participant_type: "consumer".to_owned(),
		});
		ctx.terminate();
		Running::Stop
	}
}

impl Consumer {
	pub fn new(name: String, game_id: String, game_addr: Addr<Game>) -> Consumer {
		Consumer {
			name,
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
					name: act.name.clone(),
					participant_type: "consumer".to_owned(),
				});
				if Instant::now().duration_since(act.hb) > CLIENT_TERMINATE {
					ctx.binary(
						to_vec(&ConsumerServerMsg {
							msg_type: ConsumerServerType::ServerKicked,
						})
						.unwrap(),
					);
					act.game_addr.do_send(participant_to_game::Disconnected {
						name: act.name.clone(),
						participant_type: "consumer".to_owned(),
					});
					ctx.stop();
				}
				act.is_unresponsive = true;
			}
			let ping = to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::Ping,
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
				participant_type: "consumer".to_string(),
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
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		if let Ok(ws::Message::Binary(bin)) = msg {
			if let Ok(message) = from_slice::<ConsumerClientMsg>(&bin.to_vec()) {
				match message.msg_type {
					ConsumerClientType::Choice(elements) => {
						if !self.took_turn && !self.is_producer_turn {
							// * send the message to game. game calculates purchased quantity and returns the expense, remaining balance, and total purchased quantity
							self.game_addr.do_send(consumer_to_game::TryChoice {
								name: self.name.clone(),
								elements,
							});
						}
					}
					ConsumerClientType::Pong => (),
					ConsumerClientType::EndTurn => {
						self.score += self.balance;
						self.balance = 0.;
						self.took_turn = true;
						self.game_addr.do_send(consumer_to_game::NewScoreEndTurn {
							name: self.name.clone(),
							new_score: self.score,
						});
						ctx.binary(
							to_vec(&ConsumerServerMsg {
								msg_type: ConsumerServerType::ChoiceSubmitted(
									self.balance,
									self.score,
									self.quantity_purchased,
								),
							})
							.unwrap(),
						);
						ctx.binary(
							to_vec(&ConsumerServerMsg {
								msg_type: ConsumerServerType::TurnEnded,
							})
							.unwrap(),
						);
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
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::GameEnded,
			})
			.unwrap(),
		);
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
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::NewOffsets(consumer_structs::Offsets {
					trending: msg.trending,
				}),
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
		// * if going to producer turn, update score
		if self.is_producer_turn {
			self.score += self.balance;
			self.balance = 0.;
			self.total_utility = 0.;
			self.quantity_purchased = 0.;
			ctx.binary(
				to_vec(&ConsumerServerMsg {
					msg_type: ConsumerServerType::TurnAdvanced(0., self.score),
				})
				.unwrap(),
			);
		} else {
			self.balance = INITIAL_BALANCE;
			ctx.binary(
				to_vec(&ConsumerServerMsg {
					msg_type: ConsumerServerType::TurnAdvanced(self.balance, 0.),
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
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::Info(msg.info),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_consumer::PurchaseResult> for Consumer {
	type Result = ();
	fn handle(&mut self, msg: game_to_consumer::PurchaseResult, ctx: &mut Self::Context) {
		if msg.purchased != 0. {
			self.balance = msg.balance;
			let utility = self.get_utility(msg.purchased);
			self.quantity_purchased += msg.purchased;
			self.score += utility;
			self.total_utility += utility;
			self.game_addr
				.do_send(consumer_to_game::NewScoreCalculated {
					name: self.name.clone(),
					new_score: self.score,
				});
			ctx.binary(
				to_vec(&ConsumerServerMsg {
					msg_type: ConsumerServerType::ChoiceSubmitted(
						self.balance,
						self.score,
						self.quantity_purchased,
					),
				})
				.unwrap(),
			);
		}
	}
}

impl Handler<game_to_consumer::TurnList> for Consumer {
	type Result = ();
	fn handle(&mut self, msg: game_to_consumer::TurnList, ctx: &mut Self::Context) -> Self::Result {
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::TurnInfo(consumer_structs::TurnInfo {
					producers: msg.list,
				}),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::StockReduced> for Consumer {
	type Result = ();
	fn handle(&mut self, msg: game_to_participant::StockReduced, ctx: &mut Self::Context) {
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::StockReduced(msg.targets),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::Kicked> for Consumer {
	type Result = ();
	fn handle(&mut self, _msg: game_to_participant::Kicked, ctx: &mut Self::Context) {
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::ServerKicked,
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::Winner> for Consumer {
	type Result = ();
	fn handle(&mut self, msg: game_to_participant::Winner, ctx: &mut Self::Context) {
		ctx.binary(
			to_vec(&ConsumerServerMsg {
				msg_type: ConsumerServerType::Winner(msg.hash, msg.place),
			})
			.unwrap(),
		)
	}
}
