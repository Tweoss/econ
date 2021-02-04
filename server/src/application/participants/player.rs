use actix::{/* Addr, */Actor, Context};
// use std::sync::Mutex;

// use crate::application::other_messages;
// use crate::application::game::Game; 

// //* players can make actions to be sent to the game to manage
pub struct Player {
	//* will never be modified
	pub uuid: String,
	pub username: String,
	//* will never be modified
	// game_addr: Addr<Game>,
}

impl Actor for Player {
    type Context = Context<Self>;
}

impl Player {
	pub fn new (uuid: String, username: String) -> Player{
		Player {
			uuid,
			username,
		}
	}
}