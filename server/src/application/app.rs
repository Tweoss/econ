use actix::prelude::*;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::application::app_messages::*; use crate::application::other_messages;
use crate::application::game::Game; 
// use crate::application::player::Player; 


// //* App State can receive messages about a new Game, the end of a Game, getting a Game (to operate on)

/// Actor for managing Games
    ///
    /// # Start and Send
    ///
    /// ```
    /// let addr = AppState.start();
	/// //! haven't done the messages yet
    /// let result = addr.send(app_messages::NewConnection)
    /// ```
pub struct AppState {
	game_map: Mutex<HashMap<String,Addr<Game>>>,
	game_ids: Mutex<Vec<String>>,
}

impl AppState {

	pub fn new() -> AppState {
		AppState {
			game_map: Mutex::new(HashMap::new()),
			game_ids: Mutex::new(Vec::new()),
		}
	}
	// /// check the HashMap if 
	// pub fn game_exists(&self, search_id: String) -> bool {
	// 	for id in self.game_map.lock().unwrap().keys() {
	// 		if *id == search_id {return true;}
	// 	}
	// 	false
	// }

}

impl Actor for AppState {
    type Context = actix::Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<DoesGameExist> for AppState {
    type Result = bool;
    fn handle(&mut self, msg: DoesGameExist, _: &mut Context<Self>) -> Self::Result {
		println!("msg: DoesGameExist");
		let string = msg.game_id;
        self.game_map.lock().unwrap().keys().any(|id| id == &string)
    }
}

// struct Director {
// 	//* will never be modified (randomly generated and stored in cookie)
// 	confirmation_id: usize,
// 	// //* will never be modified
// 	// addr: Addr<Director>,
// 	//* will never be modified
// 	game_addr: Addr<Game>,
// }
// struct Viewer {
// 	//* will never be modified (randomly generated and stored in cookie)
// 	confirmation_id: usize,
// 	// //* will never be modified
// 	// addr: Addr<Viewer>,
// 	//* will never be modified
// 	game_addr: Addr<Game>,
// }


