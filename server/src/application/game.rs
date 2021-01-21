use actix::{Addr, Actor, Context};
use crate::application::app::AppState; 
use crate::application::player::Player; 


// //* Game can receive messages about a Player joining, admin Messages, other things?
pub struct Game {
	//* will never be modified - read multiple times
	id: usize, //* 6 digits? 
	//* will be modified
	players: Vec<Addr<Player>>, 
	//* will never be modified
	app_addr: Addr<AppState>,
}

impl Actor for Game {
    type Context = Context<Self>;
}
