use actix::prelude::*;
use actix::StreamHandler;
use actix_web_actors::ws;
use std::time::Instant;

use crate::game_folder::game::Game;
use crate::game_folder::game_to_participant;
use crate::game_folder::game_to_producer;

use crate::participants::participant_to_game;
use crate::participants::producer_folder::producer_to_game;
use serde_cbor::{from_slice, to_vec};

use crate::participants::producer_folder::producer_structs::{
	self, ProducerClientMsg, ProducerClientType, ProducerServerMsg, ProducerServerType,
};

use crate::participants::heartbeat::{CLIENT_TERMINATE, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL};

const INITIAL_BALANCE: f64 = 4000.;
const CLIENT_T_CALCULATION_FREEDOM: f64 = 0.0001;

pub struct ProducerState {
	pub is_responsive: bool,
	pub took_turn: bool,
	pub score: f64,
	pub balance: f64,
	pub quantity_produced: f64,
	pub addr: Option<Addr<Producer>>,
	pub id: String,
}

impl ProducerState {
	pub fn new(id: String) -> ProducerState {
		ProducerState {
			is_responsive: true,
			took_turn: false,
			score: 0.,
			balance: INITIAL_BALANCE,
			quantity_produced: 0.,
			addr: None,
			id,
		}
	}
}

pub struct Producer {
	name: String,
	game_id: String,
	game_addr: Addr<Game>,
	is_producer_turn: bool,
	took_turn: bool,
	subsidies: u8,
	supply_shock: u8,
	balance: f64,
	score: f64,
	hb: Instant,
	is_unresponsive: bool,
}

impl Actor for Producer {
	type Context = ws::WebsocketContext<Self>;
	fn started(&mut self, ctx: &mut Self::Context) {
		self.game_addr
			.do_send(producer_to_game::RegisterAddressGetInfo {
				name: self.name.clone(),
				addr: ctx.address(),
			});
		self.hb(ctx);
	}
	fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
		println!(
			"Stopping a producer actor: {} and {}",
			self.game_id, self.name
		);
		self.game_addr.do_send(participant_to_game::Disconnected {
			name: self.name.clone(),
			participant_type: "producer".to_owned(),
		});
		ctx.terminate();
		Running::Stop
	}
}

impl Producer {
	pub fn new(name: String, game_id: String, game_addr: Addr<Game>) -> Producer {
		Producer {
			name,
			game_id,
			game_addr,
			subsidies: 0,
			supply_shock: 0,
			balance: 0.,
			score: 0.,
			took_turn: false,
			hb: Instant::now(),
			is_unresponsive: false,
			is_producer_turn: false,
		}
	}
	fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
		ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
			// * check client heartbeats
			if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
				// * heartbeat timed out
				// * notify game
				act.game_addr.do_send(participant_to_game::Unresponsive {
					name: act.name.clone(),
					participant_type: "producer".to_owned(),
				});
				if Instant::now().duration_since(act.hb) > CLIENT_TERMINATE {
					ctx.binary(
						to_vec(&ProducerServerMsg {
							msg_type: ProducerServerType::ServerKicked,
						})
						.unwrap(),
					);
					act.game_addr.do_send(participant_to_game::Disconnected {
						name: act.name.clone(),
						participant_type: "producer".to_owned(),
					});
					ctx.stop();
				}
				act.is_unresponsive = true;
			}
			let ping = to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::Ping,
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
				participant_type: "producer".to_string(),
			});
			self.is_unresponsive = false;
		}
	}
	#[allow(clippy::many_single_char_names)]
	// * try to produce an amount. returns score if works. if not, insufficient funds => None
	fn try_produce(&mut self, quantity: f64, t: f64, price: f64) -> Result<f64, String> {
		let believed_quantity = 3. * f64::powi(1. - t, 2) * t * 10.
			+ 3. * (1. - t) * f64::powi(t, 2) * 45.
			+ f64::powi(t, 3) * 80.;
		if believed_quantity - quantity < CLIENT_T_CALCULATION_FREEDOM
			&& believed_quantity - quantity > -CLIENT_T_CALCULATION_FREEDOM
		{
			let p = f64::from(self.supply_shock) - f64::from(self.subsidies);
			let (a, b, c, d) = (0., 10., 45., 80.);
			let (e, f, g, h) = (80., -10., -10., 100.);

			let cost =
				-3. * (a - b) * (e + p) * t
					+ (3. / 2.)
						* (5. * a * e - 7. * b * e + 2. * c * e - 3. * a * f
							+ 3. * b * f + 2. * (a - 2. * b + c) * p)
						* f64::powi(t, 2) - (9. * c * e - d * e - 6. * c * f + 3. * c * p
					- d * p - 3. * b * (6. * e - 6. * f + g + p)
					+ a * (10. * e - 12. * f + 3. * g + p))
					* f64::powi(t, 3) + (3. / 4.)
					* (3. * (5. * c * e - d * e - 7. * c * f + d * f + 2. * c * g)
						+ a * (10. * e - 18. * f + 9. * g - h)
						+ b * (-22. * e + 36. * f - 15. * g + h))
					* f64::powi(t, 4) - (3. / 5.)
					* (5. * a * e - 13. * b * e + 11. * c * e - 3. * d * e - 12. * a * f
						+ 30. * b * f - 24. * c * f
						+ 6. * d * f + 9. * a * g
						- 21. * b * g + 15. * c * g
						- 3. * d * g - 2. * (a - 2. * b + c) * h)
					* f64::powi(t, 5) + (1. / 2.)
					* (a - 3. * b + 3. * c - d)
					* (e - 3. * f + 3. * g - h)
					* f64::powi(t, 6);
			if cost > self.balance {
				Err("Insufficient Funds".to_string())
			} else {
				//* no immediate gain for producers
				self.balance -= cost;
				//* remove the rest of the balance and put as part of score
				self.score += self.balance;
				self.balance = 0.;
				self.game_addr.do_send(producer_to_game::NewScoreEndTurn {
					new_score: self.score,
					name: self.name.clone(),
					produced: quantity,
					price,
				});
				Ok(self.score)
			}
		} else {
			Err("Inaccurate t Value".to_string())
		}
	}
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Producer {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		if let Ok(ws::Message::Binary(bin)) = msg {
			if let Ok(message) = from_slice::<ProducerClientMsg>(&bin.to_vec()) {
				match message.msg_type {
					ProducerClientType::Choice(choice) => {
						if !self.took_turn && self.is_producer_turn {
							match self.try_produce(choice.quantity, choice.t, choice.price) {
								Ok(score) => {
									ctx.binary(
										to_vec(&ProducerServerMsg {
											msg_type: ProducerServerType::ChoiceSubmitted(
												score,
												self.balance,
											),
										})
										.unwrap(),
									);
									self.took_turn = true;
								}
								Err(msg) => {
									ctx.binary(
										to_vec(&ProducerServerMsg {
											msg_type: ProducerServerType::ChoiceFailed(msg),
										})
										.unwrap(),
									);
								}
							}
						}
					}
					ProducerClientType::Pong => (),
				}
			} else {
				println!("Invalid structure received");
			}
		}
		self.reset_hb();
	}
}

impl Handler<game_to_participant::EndedGame> for Producer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::EndedGame,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::GameEnded,
			})
			.unwrap(),
		);
		ctx.stop();
	}
}

impl Handler<game_to_participant::NewOffsets> for Producer {
	type Result = ();
	fn handle(&mut self, msg: game_to_participant::NewOffsets, ctx: &mut Self::Context) {
		ctx.binary(
			to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::NewOffsets(producer_structs::Offsets {
					subsidies: msg.subsidies,
					supply_shock: msg.supply_shock,
				}),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::Kicked> for Producer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::Kicked,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::ServerKicked,
			})
			.unwrap(),
		);
		ctx.terminate();
	}
}

impl Handler<game_to_participant::TurnAdvanced> for Producer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::TurnAdvanced,
		ctx: &mut Self::Context,
	) -> Self::Result {
		self.took_turn = false;
		self.is_producer_turn = !self.is_producer_turn;
		// * if going to consumer turn, update score
		if !self.is_producer_turn {
			self.score += self.balance;
			self.balance = 0.;
			ctx.binary(
				to_vec(&ProducerServerMsg {
					msg_type: ProducerServerType::TurnAdvanced(0., self.score),
				})
				.unwrap(),
			);
		} else {
			self.balance = INITIAL_BALANCE;
			ctx.binary(
				to_vec(&ProducerServerMsg {
					msg_type: ProducerServerType::TurnAdvanced(self.balance, 0.),
				})
				.unwrap(),
			);
		}
	}
}

impl Handler<game_to_participant::StockReduced> for Producer {
	type Result = ();
	fn handle(&mut self, msg: game_to_participant::StockReduced, ctx: &mut Self::Context) {
		ctx.binary(
			to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::StockReduced(msg.targets),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_producer::Info> for Producer {
	type Result = ();
	fn handle(&mut self, msg: game_to_producer::Info, ctx: &mut Self::Context) -> Self::Result {
		self.subsidies = msg.info.subsidies;
		self.supply_shock = msg.info.supply_shock;
		self.balance = msg.info.balance;
		self.score = msg.info.score;
		self.took_turn = msg.info.took_turn;
		self.is_producer_turn = msg.info.turn % 2 == 1;
		ctx.binary(
			to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::Info(msg.info),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_producer::TurnList> for Producer {
	type Result = ();
	fn handle(&mut self, msg: game_to_producer::TurnList, ctx: &mut Self::Context) -> Self::Result {
		ctx.binary(
			to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::TurnInfo(producer_structs::TurnInfo {
					producers: msg.list,
				}),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_producer::GotPurchased> for Producer {
	type Result = ();
	fn handle(
		&mut self,
		msg: game_to_producer::GotPurchased,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.binary(
			to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::GotPurchased(msg.additional_score),
			})
			.unwrap(),
		);
	}
}

impl Handler<game_to_participant::Winner> for Producer {
	type Result = ();
	fn handle(&mut self, msg: game_to_participant::Winner, ctx: &mut Self::Context) {
		ctx.binary(
			to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::Winner(msg.hash, msg.place),
			})
			.unwrap(),
		)
	}
}
