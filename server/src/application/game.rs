use actix::prelude::*;
use crate::application::app::AppState; 
use super::participants::player::Player;
use crate::application::app_to_game::*;

use std::sync::Mutex;
use std::collections::HashMap;


// //* Game can receive messages about a Player joining, admin Messages, other things?
pub struct Game {
	//* will never be modified - read multiple times
	// id: usize, //* 6 digits? 
	//* will be modified
	producers: Mutex<HashMap<String, Option<Addr<Player>>>>,
	consumers: Mutex<HashMap<String, Option<Addr<Player>>>>,
	directors: Mutex<HashMap<String, Option<Addr<Player>>>>,
	id_main_director: String,
	addr_main_director: Option<Addr<Player>>,
	is_open: bool,
	// // * will never be modified
	app_addr: Addr<AppState>,
	producer_next: bool
}

impl Actor for Game {
    type Context = Context<Self>;
}

impl Game {
	pub fn new(app_addr: Addr<AppState>, id_main_director: String) -> Game {
		Game {
			producers: Mutex::new(HashMap::new()),
			consumers: Mutex::new(HashMap::new()),
			directors: Mutex::new(HashMap::new()),
			id_main_director,
			addr_main_director: None,
			is_open: false,
			app_addr,
			producer_next: true,
		}
	}
}


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
		}
		else {
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