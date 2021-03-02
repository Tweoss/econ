use super::participants::{
	consumer_folder::consumer::ConsumerState, director_folder::{director::DirectorState, director_structs},
	producer_folder::producer::ProducerState, viewer_folder::viewer::ViewerState,
};
use actix::prelude::*;

use crate::application::app::AppState;
use crate::application::app_to_game::*;
use crate::application::game_folder::game_to_director;
use crate::application::game_folder::game_to_participant;
use crate::application::game_folder::participants::director_folder::director_to_game;
use crate::application::game_folder::participants::participant_to_game;

use super::participants::json;

use crate::application::game_folder::game_to_app;
use std::collections::HashMap;
use std::sync::Mutex;

// //* Game can receive messages about a Player joining, admin Messages, other things?

pub struct Game {
	// is_connected, i64: score in dollars
	consumers: Mutex<HashMap<String, ConsumerState>>,
	// is_connected, i64: score in dollars, u64 is price, u64 is quantity
	producers: Mutex<HashMap<String, ProducerState>>,
	directors: Mutex<HashMap<String, DirectorState>>,
	viewers: Mutex<HashMap<String, ViewerState>>,
	id_main_director: String,
	state_main_director: DirectorState,
	is_open: bool,
	turn: u64,
	// // * will never be modified
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
			producers: Mutex::new(HashMap::new()),
			consumers: Mutex::new(HashMap::new()),
			directors: Mutex::new(HashMap::new()),
			viewers: Mutex::new(HashMap::new()),
			id_main_director,
			state_main_director: DirectorState::new(),
			is_open: false,
			turn: 0,
			app_addr,
			producer_next: true,
			game_id,
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
				.lock()
				.unwrap()
				.insert(msg.user_id.clone(), ConsumerState::new());
			for elem in self.directors.lock().unwrap().values() {
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
				.lock()
				.unwrap()
				.insert(msg.user_id.clone(), ProducerState::new());
			for elem in self.directors.lock().unwrap().values() {
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
			.lock()
			.unwrap()
			.insert(msg.user_id.clone(), DirectorState::new());
		for elem in self.directors.lock().unwrap().values() {
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
		self.directors.lock().unwrap().contains_key(&msg.user_id)
			|| self.id_main_director == msg.user_id
	}
}

/// Check if this consumer or producer is registered
impl Handler<IsPlayer> for Game {
	type Result = bool;
	fn handle(&mut self, msg: IsPlayer, _: &mut Context<Self>) -> Self::Result {
		self.consumers.lock().unwrap().contains_key(&msg.user_id)
			|| self.producers.lock().unwrap().contains_key(&msg.user_id)
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
		for director in self.directors.lock().unwrap().values() {
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

impl Handler<director_to_game::RegisterAddress> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: director_to_game::RegisterAddress,
		_: &mut Context<Self>,
	) -> Self::Result {
		if msg.user_id == self.id_main_director {
			self.state_main_director.addr = Some(msg.addr);
		} else if let Some(mut addr_value) = self.directors.lock().unwrap().get_mut(&msg.user_id) {
			addr_value.addr = Some(msg.addr);
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
		self.consumers.lock().unwrap().remove(&msg.user_id);
		self.producers.lock().unwrap().remove(&msg.user_id);
		self.directors.lock().unwrap().remove(&msg.user_id);
		self.viewers.lock().unwrap().remove(&msg.user_id);
		for elem in self.directors.lock().unwrap().values() {
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
	fn handle(&mut self, msg: director_to_game::OpenGame, _: &mut Context<Self>) {
		self.is_open = true;
		for elem in self.directors.lock().unwrap().values() {
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
	fn handle(&mut self, msg: director_to_game::CloseGame, _: &mut Context<Self>) {
		self.is_open = false;
		for elem in self.directors.lock().unwrap().values() {
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
		for elem in self.directors.lock().unwrap().values() {
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
