use super::participants::{
	consumer::Consumer, director::Director, producer::Producer, viewer::Viewer,
	ws_to_game::*
};
use crate::application::app::AppState;
use crate::application::app_to_game::*;
use actix::prelude::*;

use std::collections::HashMap;
use std::sync::Mutex;

// //* Game can receive messages about a Player joining, admin Messages, other things?
pub struct Game {
	//* will never be modified - read multiple times
	// id: usize, //* 6 digits?
	//* will be modified
	producers: Mutex<HashMap<String, Option<Addr<Producer>>>>,
	consumers: Mutex<HashMap<String, Option<Addr<Consumer>>>>,
	directors: Mutex<HashMap<String, Option<Addr<Director>>>>,
	viewers: Mutex<HashMap<String, Option<Addr<Viewer>>>>,
	id_main_director: String,
	addr_main_director: Option<Addr<Director>>,
	is_open: bool,
	// // * will never be modified
	app_addr: Addr<AppState>,
	producer_next: bool,
}

impl Actor for Game {
	type Context = Context<Self>;
	fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
		println!("TRIED TO STOP");
		Running::Continue
	}
}

impl Game {
	pub fn new(app_addr: Addr<AppState>, id_main_director: String) -> Game {
		println!("Making a new GAME with director id: {}", id_main_director);
		Game {
			producers: Mutex::new(HashMap::new()),
			consumers: Mutex::new(HashMap::new()),
			directors: Mutex::new(HashMap::new()),
			viewers: Mutex::new(HashMap::new()),
			id_main_director,
			addr_main_director: None,
			is_open: false,
			app_addr,
			producer_next: true,
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
			self.consumers.lock().unwrap().insert(msg.user_id, None);
			"consumer".to_string()
		} else {
			self.producers.lock().unwrap().insert(msg.user_id, None);
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
		self.directors.lock().unwrap().insert(msg.user_id, None);
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