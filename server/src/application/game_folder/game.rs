use super::participants::{
	consumer_folder::consumer::ConsumerState,
	director_folder::{director::DirectorState, director_structs},
	producer_folder::{producer::ProducerState, producer_structs},
	viewer_folder::viewer::ViewerState,
};
// use crate::application::game_folder::participants::director_folder::director_structs;
use actix::prelude::*;

use crate::application::app::AppState;
use crate::application::app_to_game::*;
use crate::application::game_folder::game_to_director;
use crate::application::game_folder::game_to_participant;
use crate::application::game_folder::game_to_producer;
use crate::application::game_folder::participants::director_folder::director_to_game;
use crate::application::game_folder::participants::participant_to_game;
use crate::application::game_folder::participants::producer_folder::producer_to_game;

use crate::application::game_folder::game_to_app;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct Game {
	// is_connected, i64: score in dollars
	consumers: RwLock<HashMap<String, ConsumerState>>,
	// is_connected, i64: score in dollars, u64 is price, u64 is quantity
	producers: RwLock<HashMap<String, ProducerState>>,
	directors: RwLock<HashMap<String, DirectorState>>,
	viewers: RwLock<HashMap<String, ViewerState>>,
	past_turn: RwLock<Vec<(String, producer_structs::Participant)>>,
	id_main_director: String,
	state_main_director: DirectorState,
	is_open: bool,
	turn: u64,
	trending: u8,
	supply_shock: u8,
	subsidies: u8,
	app_addr: Addr<AppState>,
	producer_next: bool,
	game_id: String,
}

impl Actor for Game {
	type Context = Context<Self>;
	fn stopping(&mut self, _: &mut Self::Context) -> Running {
		println!(
			"Stopping a game actor with main director being: {} and id: {}",
			self.id_main_director, self.game_id
		);
		let date = chrono::Local::now();
		println!("Date and time: {}", date.format("[%Y-%m-%d][%H:%M:%S]"));
		Running::Stop
	}
}

impl Game {
	pub fn new(app_addr: Addr<AppState>, id_main_director: String, name_main_director: String, game_id: String) -> Game {
		println!("Making a new GAME with director id: {}", id_main_director);
		Game {
			producers: RwLock::new(HashMap::new()),
			consumers: RwLock::new(HashMap::new()),
			directors: RwLock::new(HashMap::new()),
			viewers: RwLock::new(HashMap::new()),
			past_turn: RwLock::new(Vec::new()),
			id_main_director,
			state_main_director: DirectorState::new(name_main_director),
			is_open: false,
			trending: 10,
			supply_shock: 10,
			subsidies: 0,
			turn: 1,
			app_addr,
			producer_next: true,
			game_id,
		}
	}
	fn get_director_info(&self) -> director_structs::Info {
		let mut consumers = Vec::new();
		let mut producers = Vec::new();
		let mut directors = Vec::new();
		let mut viewers = Vec::new();

		for consumer in self.consumers.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if consumer.1.addr != None {
				if consumer.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				} else {
					state = director_structs::PlayerState::Unresponsive;
				}
			} else {
				state = director_structs::PlayerState::Disconnected;
			}
			consumers.push((
				consumer.0.clone(),
				director_structs::Participant {
					state,
					took_turn: Some(consumer.1.took_turn),
					name: consumer.1.name.clone(),
				},
			))
		}

		for producer in self.producers.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if producer.1.addr != None {
				if producer.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				} else {
					state = director_structs::PlayerState::Unresponsive;
				}
			} else {
				state = director_structs::PlayerState::Disconnected;
			}
			producers.push((
				producer.0.clone(),
				director_structs::Participant {
					state,
					took_turn: Some(producer.1.took_turn),
					name: producer.1.name.clone(),
				},
			))
		}

		for viewer in self.viewers.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if viewer.1.addr != None {
				if viewer.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				} else {
					state = director_structs::PlayerState::Unresponsive;
				}
			} else {
				state = director_structs::PlayerState::Disconnected;
			}
			viewers.push((
				viewer.0.clone(),
				director_structs::Participant {
					state,
					took_turn: None,
					name: viewer.1.name.clone(),
				},
			))
		}

		for director in self.directors.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if director.1.addr != None {
				if director.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				} else {
					state = director_structs::PlayerState::Unresponsive;
				}
			} else {
				state = director_structs::PlayerState::Disconnected;
			}
			directors.push((
				director.0.clone(),
				director_structs::Participant {
					state,
					took_turn: None,
					name: director.1.name.clone(),
				},
			))
		}
		director_structs::Info {
			consumers,
			producers,
			directors,
			viewers,
			is_open: self.is_open,
			turn: self.turn,
			trending: self.trending,
			supply_shock: self.supply_shock,
			subsidies: self.subsidies,
			game_id: self.game_id.clone(),
		}
	}
	fn get_producer_info(&self, id: String) -> producer_structs::Info {
		let mut producers = Vec::new();
		for producer in self.past_turn.read().unwrap().iter() {
			producers.push((producer.0.clone(), producer.1.clone()));
		}
		// let score = self.producers.read().unwrap().get(&id).unwrap().score;
		let turn = self.turn;
		let game_id = self.game_id.clone();
		let supply_shock = self.supply_shock;
		let subsidies = self.subsidies;
		let producer = self.producers.read().unwrap();
		let balance = producer.get(&id).unwrap().balance;
		let score = producer.get(&id).unwrap().score;
		let took_turn = producer.get(&id).unwrap().took_turn;
		producer_structs::Info {
			producers,
			turn,
			game_id,
			supply_shock,
			subsidies,
			balance,
			score,
			took_turn,
		}
	}
	// fn next_turn(&self)
}

// ! APPLICATION TO GAME HANDLERS

/// Handler for NewPlayer message.
///
/// Register a NewPlayer
impl Handler<NewPlayer> for Game {
	type Result = String;
	fn handle(&mut self, msg: NewPlayer, _: &mut Context<Self>) -> Self::Result {
		self.producer_next = !self.producer_next;
		if self.producer_next {
			self.consumers.write().unwrap().insert(
				msg.user_id.clone(),
				ConsumerState::new(msg.username.clone()),
			);
			for elem in self.directors.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_director::NewParticipant {
						id: msg.user_id.clone(),
						name: msg.username.clone(),
						participant_type: director_structs::ParticipantType::Consumer,
					});
				}
			}
			if let Some(addr) = &self.state_main_director.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id,
					name: msg.username,
					participant_type: director_structs::ParticipantType::Consumer,
				})
			}
			"consumer".to_string()
		} else {
			self.producers.write().unwrap().insert(
				msg.user_id.clone(),
				ProducerState::new(msg.username.clone()),
			);
			for elem in self.directors.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_director::NewParticipant {
						id: msg.user_id.clone(),
						name: msg.username.clone(),
						participant_type: director_structs::ParticipantType::Producer,
					});
				}
			}
			if let Some(addr) = &self.state_main_director.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id,
					name: msg.username,
					participant_type: director_structs::ParticipantType::Producer,
				})
			}
			"producer".to_string()
		}
	}
}

/// Return if the Game is open to players
impl Handler<IsGameOpen> for Game {
	type Result = bool;
	fn handle(&mut self, _: IsGameOpen, _: &mut Context<Self>) -> Self::Result {
		self.is_open
	}
}

/// Register an additional director
impl Handler<NewDirector> for Game {
	type Result = ();
	fn handle(&mut self, msg: NewDirector, _: &mut Context<Self>) -> Self::Result {
		self.directors
			.write()
			.unwrap()
			.insert(msg.user_id.clone(), DirectorState::new(msg.username.clone()));
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id.clone(),
					name: msg.username.clone(),
					participant_type: director_structs::ParticipantType::Director,
				});
			}
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::NewParticipant {
				id: msg.user_id,
				name: msg.username,
				participant_type: director_structs::ParticipantType::Director,
			});
		}
	}
}

/// Check if this director is registered
impl Handler<IsDirector> for Game {
	type Result = bool;
	fn handle(&mut self, msg: IsDirector, _: &mut Context<Self>) -> Self::Result {
		println!("Asked if IsDirector for a game.");
		self.directors.read().unwrap().contains_key(&msg.user_id)
			|| self.id_main_director == msg.user_id
	}
}

/// Check if this consumer or producer is registered
impl Handler<IsPlayer> for Game {
	type Result = bool;
	fn handle(&mut self, msg: IsPlayer, _: &mut Context<Self>) -> Self::Result {
		self.consumers.read().unwrap().contains_key(&msg.user_id)
			|| self.producers.read().unwrap().contains_key(&msg.user_id)
	}
}

impl Handler<IsMainDirector> for Game {
	type Result = bool;
	fn handle(&mut self, msg: IsMainDirector, _: &mut Context<Self>) -> Self::Result {
		self.id_main_director == msg.user_id
	}
}

// ! WEBSOCKET TO GAME HANDLERS

impl Handler<director_to_game::EndGame> for Game {
	type Result = ();
	fn handle(&mut self, _msg: director_to_game::EndGame, ctx: &mut Context<Self>) -> Self::Result {
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_participant::EndedGame {});
		}
		for director in self.directors.read().unwrap().values() {
			if let Some(addr) = &director.addr {
				addr.do_send(game_to_participant::EndedGame {});
			}
		}
		self.app_addr.do_send(game_to_app::EndGame {
			game_id: self.game_id.clone(),
		});
		ctx.stop();
	}
}

impl Handler<director_to_game::RegisterAddressGetInfo> for Game {
	// type Result = MessageResult<director_structs::Info>;
	type Result = ();
	fn handle(
		&mut self,
		msg: director_to_game::RegisterAddressGetInfo,
		_: &mut Context<Self>,
	) -> Self::Result {
		if msg.user_id == self.id_main_director {
			self.state_main_director.addr = Some(msg.addr.clone());
		} else if let Some(mut addr_value) = self.directors.write().unwrap().get_mut(&msg.user_id) {
			addr_value.addr = Some(msg.addr.clone());
		}
		msg.addr.do_send(game_to_director::Info {
			info: self.get_director_info(),
		});
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Connected {
				id: msg.user_id.clone(),
				participant_type: "director".to_string(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Connected {
					id: msg.user_id.clone(),
					participant_type: "director".to_string(),
				});
			}
		}
	}
}

impl Handler<director_to_game::KickParticipant> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: director_to_game::KickParticipant,
		_: &mut Context<Self>,
	) -> Self::Result {
		self.consumers.write().unwrap().remove(&msg.user_id);
		self.producers
			.write()
			.unwrap()
			.remove_entry(&msg.user_id)
			.map(|x| {
				x.1.addr
					.map(|addr| addr.do_send(game_to_participant::Kicked {}))
			});
		// self.producers.write().unwrap().remove(&msg.user_id);
		self.directors
			.write()
			.unwrap()
			.remove_entry(&msg.user_id)
			.map(|x| {
				x.1.addr
					.map(|addr| addr.do_send(game_to_participant::Kicked {}))
			});
		// self.directors.write().unwrap().remove(&msg.user_id);
		self.viewers.write().unwrap().remove(&msg.user_id);
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::KickedParticipant {
					id: msg.user_id.clone(),
				});
			};
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::KickedParticipant { id: msg.user_id });
		}
	}
}

impl Handler<director_to_game::OpenGame> for Game {
	type Result = ();
	fn handle(&mut self, _msg: director_to_game::OpenGame, _: &mut Context<Self>) {
		self.is_open = true;
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::GameOpened {});
			}
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::GameOpened {});
		}
	}
}

impl Handler<director_to_game::CloseGame> for Game {
	type Result = ();
	fn handle(&mut self, _msg: director_to_game::CloseGame, _: &mut Context<Self>) {
		self.is_open = false;
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::GameClosed {});
			}
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::GameClosed {});
		}
	}
}

impl Handler<director_to_game::SetOffsets> for Game {
	type Result = ();
	fn handle(&mut self, msg: director_to_game::SetOffsets, _: &mut Context<Self>) {
		// * Turn starts at 1, producer.
		if self.turn % 2 == 1
			&& self.subsidies == msg.subsidies
			&& self.supply_shock == msg.supply_shock
			|| self.turn % 2 == 0 && self.trending == msg.trending
		{
			// * make sure that the others aren't changing
			self.subsidies = msg.subsidies;
			self.trending = msg.trending;
			self.supply_shock = msg.supply_shock;
			if let Some(addr) = &self.state_main_director.addr {
				addr.do_send(game_to_participant::NewOffsets {
					subsidies: msg.subsidies,
					trending: msg.trending,
					supply_shock: msg.supply_shock,
				});
			}
			for elem in self.directors.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_participant::NewOffsets {
						subsidies: msg.subsidies,
						trending: msg.trending,
						supply_shock: msg.supply_shock,
					});
				}
			}
			for elem in self.producers.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_participant::NewOffsets {
						subsidies: msg.subsidies,
						trending: msg.trending,
						supply_shock: msg.supply_shock,
					});
				}
			}
		}
		// for elem in self.consumers.read().unwrap().values() {
		// 	if let Some(addr) = &elem.addr {
		// 		addr.do_send(game_to_participant::NewOffsets {subsidies: msg.subsidies, trending: msg.trending, supply_shock: msg.supply_shock});
		// 	}
		// }
	}
}

impl Handler<director_to_game::ForceTurn> for Game {
	type Result = ();
	fn handle(&mut self, _: director_to_game::ForceTurn, _: &mut Context<Self>) -> Self::Result {
		self.turn += 1;
		if self.turn % 2 == 0 {
			let list = self.past_turn.read().unwrap().clone();
			for elem in self.producers.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_producer::TurnList { list: list.clone() });
				}
			}
			for producer in self.producers.write().unwrap().values_mut() {
				producer.took_turn = false;
			}
			for consumer in self.consumers.write().unwrap().values_mut() {
				consumer.took_turn = false;
			}
			// self.producers.write().unwrap().values_mut().map(|elem| elem.took_turn = false);
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_participant::TurnAdvanced {});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_participant::TurnAdvanced {});
			}
		}
		for elem in self.producers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_participant::TurnAdvanced {});
			}
		}
	}
}

impl Handler<producer_to_game::RegisterAddressGetInfo> for Game {
	// type Result = MessageResult<director_structs::Info>;
	type Result = ();
	fn handle(
		&mut self,
		msg: producer_to_game::RegisterAddressGetInfo,
		_: &mut Context<Self>,
	) -> Self::Result {
		if let Some(mut addr_value) = self.producers.write().unwrap().get_mut(&msg.user_id) {
			addr_value.addr = Some(msg.addr.clone());
		}
		msg.addr.do_send(game_to_producer::Info {
			info: self.get_producer_info(msg.user_id.clone()),
		});
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Connected {
				id: msg.user_id.clone(),
				participant_type: "producer".to_string(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Connected {
					id: msg.user_id.clone(),
					participant_type: "producer".to_string(),
				});
			}
		}
	}
}

impl Handler<producer_to_game::NewScoreEndTurn> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: producer_to_game::NewScoreEndTurn,
		_: &mut Context<Self>,
	) -> Self::Result {
		self.producers
			.write()
			.unwrap()
			.get_mut(&msg.user_id)
			.unwrap()
			.score = msg.new_score;
		self.producers
			.write()
			.unwrap()
			.get_mut(&msg.user_id)
			.unwrap()
			.balance = 0.;
		self.producers
			.write()
			.unwrap()
			.get_mut(&msg.user_id)
			.unwrap()
			.took_turn = true;
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::TurnTaken {
				id: msg.user_id.clone(),
				participant_type: "producer".to_string(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::TurnTaken {
					id: msg.user_id.clone(),
					participant_type: "producer".to_string(),
				});
			}
		}
		// for elem in self.viewers.read().unwrap().values() {
		// 	if let Some(addr) = &elem.addr {
		// 		addr.do_send(game_to_viewer::NewScoreEndTurn {
		// 			id: msg.user_id.clone(),
		// 			participant_type: "producer".to_string(),
		// 			new_score: msg.new_score,
		// 		});
		// 	}
		// }
		self.past_turn.write().unwrap().push((
			msg.user_id,
			producer_structs::Participant {
				produced: msg.produced,
				remaining: msg.produced,
				price: msg.price,
			},
		));
	}
}

impl Handler<participant_to_game::Unresponsive> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: participant_to_game::Unresponsive,
		_: &mut Context<Self>,
	) -> Self::Result {
		println!("Unresponsive {}: {}", &msg.participant_type, &msg.id);
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Unresponsive {
				id: msg.id.clone(),
				participant_type: msg.participant_type.clone(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Unresponsive {
					id: msg.id.clone(),
					participant_type: msg.participant_type.clone(),
				});
			}
		}
	}
}

impl Handler<participant_to_game::Responsive> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: participant_to_game::Responsive,
		_: &mut Context<Self>,
	) -> Self::Result {
		println!("Responsive again");
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Connected {
				id: msg.id.clone(),
				participant_type: msg.participant_type.clone(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Connected {
					id: msg.id.clone(),
					participant_type: msg.participant_type.clone(),
				});
			}
		}
	}
}

impl Handler<participant_to_game::Disconnected> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: participant_to_game::Disconnected,
		_: &mut Context<Self>,
	) -> Self::Result {
		match msg.participant_type.as_str() {
			"director" => {
				if msg.id == self.id_main_director {
					self.state_main_director.addr = None;
				} else {
					self.directors
						.write()
						.unwrap()
						.get_mut(&msg.id)
						.unwrap()
						.addr = None;
				}
			}
			"consumer" => {
				self.consumers
					.write()
					.unwrap()
					.get_mut(&msg.id)
					.unwrap()
					.addr = None;
			}
			"producer" => {
				self.producers
					.write()
					.unwrap()
					.get_mut(&msg.id)
					.unwrap()
					.addr = None;
			}
			"viewer" => {
				self.viewers.write().unwrap().get_mut(&msg.id).unwrap().addr = None;
			}
			_ => (),
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Disconnected {
				id: msg.id.clone(),
				participant_type: msg.participant_type.clone(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Disconnected {
					id: msg.id.clone(),
					participant_type: msg.participant_type.clone(),
				});
			}
		}
	}
}

// /// Check if this consumer is registered
// impl Handler<IsConsumer> for Game {
// 	type Result = bool;
// 	fn handle(&mut self, msg: IsPlayer, _: &mut Context<Self>) -> Self::Result {
// 		self.consumers.lock().unwrap().contains_key(&msg.user_id)
// 			|| self.producers.lock().unwrap().contains_key(&msg.user_id)
// 	}
// }

// /// Check if this consumer is registered
// impl Handler<IsProducerr> for Game {
// 	type Result = bool;
// 	fn handle(&mut self, msg: IsPlayer, _: &mut Context<Self>) -> Self::Result {
// 		self.consumers.lock().unwrap().contains_key(&msg.user_id)
// 			|| self.producers.lock().unwrap().contains_key(&msg.user_id)
// 	}
// }

// /// Check if this consumer is registered
// impl Handler<IsViewer> for Game {
// 	type Result = bool;
// 	fn handle(&mut self, msg: IsPlayer, _: &mut Context<Self>) -> Self::Result {
// 		self.consumers.lock().unwrap().contains_key(&msg.user_id)
// 			|| self.producers.lock().unwrap().contains_key(&msg.user_id)
// 	}
// }
