use super::participants::{
	consumer_folder::consumer::ConsumerState, director_folder::{director::DirectorState, director_structs},
	producer_folder::producer::ProducerState, viewer_folder::viewer::ViewerState,
};
// use crate::application::game_folder::participants::director_folder::director_structs;
use actix::prelude::*;

use crate::application::app::AppState;
use crate::application::app_to_game::*;
use crate::application::game_folder::game_to_director;
use crate::application::game_folder::game_to_participant;
use crate::application::game_folder::participants::director_folder::director_to_game;
use crate::application::game_folder::participants::participant_to_game;

// use super::participants::json;

use crate::application::game_folder::game_to_app;
use std::collections::HashMap;
use std::sync::RwLock;

// //* Game can receive messages about a Player joining, admin Messages, other things?

pub struct Game {
	// is_connected, i64: score in dollars
	consumers: RwLock<HashMap<String, ConsumerState>>,
	// is_connected, i64: score in dollars, u64 is price, u64 is quantity
	producers: RwLock<HashMap<String, ProducerState>>,
	directors: RwLock<HashMap<String, DirectorState>>,
	viewers: RwLock<HashMap<String, ViewerState>>,
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
			"Stopping a game actor with main director being: {}",
			self.id_main_director
		);
		Running::Stop
	}
}

impl Game {
	pub fn new(app_addr: Addr<AppState>, id_main_director: String, game_id: String) -> Game {
		println!("Making a new GAME with director id: {}", id_main_director);
		Game {
			producers: RwLock::new(HashMap::new()),
			consumers: RwLock::new(HashMap::new()),
			directors: RwLock::new(HashMap::new()),
			viewers: RwLock::new(HashMap::new()),
			id_main_director,
			state_main_director: DirectorState::new(),
			is_open: false,
			trending: 10,
			supply_shock: 10,
			subsidies: 0,
			turn: 0,
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
				}
				else {
					state = director_structs::PlayerState::Unresponsive;
				}
			}
			else {
				state = director_structs::PlayerState::Disconnected;
			}
			consumers.push((consumer.0.clone(), director_structs::Participant {
				state,
				took_turn: Some(consumer.1.took_turn)
			}))
		}

		for producer in self.producers.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if producer.1.addr != None {
				if producer.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				}
				else {
					state = director_structs::PlayerState::Unresponsive;
				}
			}
			else {
				state = director_structs::PlayerState::Disconnected;
			}
			producers.push((producer.0.clone(), director_structs::Participant {
				state,
				took_turn: Some(producer.1.took_turn)
			}))
		}

		for viewer in self.viewers.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if viewer.1.addr != None {
				if viewer.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				}
				else {
					state = director_structs::PlayerState::Unresponsive;
				}
			}
			else {
				state = director_structs::PlayerState::Disconnected;
			}
			viewers.push((viewer.0.clone(), director_structs::Participant {
				state,
				took_turn: None,
			}))
		}

		for director in self.directors.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if director.1.addr != None {
				if director.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				}
				else {
					state = director_structs::PlayerState::Unresponsive;
				}
			}
			else {
				state = director_structs::PlayerState::Disconnected;
			}
			directors.push((director.0.clone(), director_structs::Participant {
				state,
				took_turn: None,
			}))
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
			self.consumers
				.write()
				.unwrap()
				.insert(msg.user_id.clone(), ConsumerState::new());
			for elem in self.directors.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_director::NewParticipant {
						id: msg.user_id.clone(),
						participant_type: director_structs::ParticipantType::Consumer,
					});
				}
			}
			if let Some(addr) = &self.state_main_director.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id,
					participant_type: director_structs::ParticipantType::Consumer,
				})
			}
			"consumer".to_string()
		} else {
			self.producers
				.write()
				.unwrap()
				.insert(msg.user_id.clone(), ProducerState::new());
			for elem in self.directors.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_director::NewParticipant {
						id: msg.user_id.clone(),
						participant_type: director_structs::ParticipantType::Producer,
					});
				}
			}
			if let Some(addr) = &self.state_main_director.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id,
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
			.insert(msg.user_id.clone(), DirectorState::new());
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id.clone(),
					participant_type: director_structs::ParticipantType::Director,
				});
			}
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::NewParticipant {
				id: msg.user_id,
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
		msg.addr.do_send(game_to_director::Info {info: self.get_director_info()});
		// MessageResult(self.get_director_info())
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
		self.producers.write().unwrap().remove(&msg.user_id);
		self.directors.write().unwrap().remove(&msg.user_id);
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

impl Handler<participant_to_game::Unresponsive> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: participant_to_game::Unresponsive,
		_: &mut Context<Self>,
	) -> Self::Result {
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Unresponsive { id: msg.id.clone() });
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Unresponsive { id: msg.id.clone() });
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
