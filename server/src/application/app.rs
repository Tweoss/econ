use actix::prelude::*;
use std::sync::Mutex;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};

use crate::application::handle_to_app::*; use crate::application::app_to_game; use crate::application::participants::ws_to_app;
use crate::application::game::Game; 
// use crate::application::player::Player; 


// //* App State can receive messages about a new Game, the end of a Game, getting a Game (to operate on)

/// Actor for managing Games
    ///
    /// # Start and Send
    ///
    /// ```
    /// let addr = AppState.start();
    /// let result = addr.send(handle_to_app::DoesGameExist {game_id} )
    /// ```
pub struct AppState {
	game_map: Mutex<HashMap<String,Addr<Game>>>,
	// game_ids: Mutex<Vec<String>>,
	password_hash: u64,
    temp_addr: Option<Addr<Game>>,
}

impl AppState {
	pub fn new() -> AppState {
		AppState {
			game_map: Mutex::new(HashMap::new()),
			// game_ids: Mutex::new(Vec::new()),
			password_hash: 9612577385432581406,
            temp_addr: None,
		}
	}
}

impl Actor for AppState {
    type Context = actix::Context<Self>;
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        println!("APP TRIED TO STOP");
        Running::Continue
    }
}

/// Handler for DoesGameExist message.
///
/// Check if a game_id is valid
impl Handler<DoesGameExist> for AppState {
    type Result = bool;
    fn handle(&mut self, msg: DoesGameExist, _: &mut Context<Self>) -> Self::Result {
		println!("msg: DoesGameExist");
		let string = msg.game_id;
        self.game_map.lock().unwrap().keys().any(|id| id == &string)
    }
}

/// Handler for IsRightPswd message.
///
/// Check if the Director submitted the correct pswd
impl Handler<IsRightPswd> for AppState {
    type Result = bool;
    fn handle(&mut self, msg: IsRightPswd, _: &mut Context<Self>) -> Self::Result {
		println!("msg: IsRightPswd");
		let mut hasher = DefaultHasher::new();
		println!("Hash is {:x}!", hasher.finish());
		msg.pswd.hash(&mut hasher);
		hasher.finish() == self.password_hash
    }
}

/// Handler for NewPlayer message.
///
/// Ask the Game to register a new player
impl Handler<NewPlayer> for AppState {
    type Result = ResponseFuture<String>;
    fn handle(&mut self, msg: NewPlayer,  _: &mut Context<Self>) -> Self::Result {
        let game_addr = self.game_map.lock().unwrap().get(&msg.game_id).unwrap().clone();
		Box::pin(async move {
			game_addr.send(app_to_game::NewPlayer {user_id: msg.user_id, username: msg.username}).await.unwrap()
		})
    }
}

impl Handler<IsGameOpen> for AppState {
    type Result = ResponseFuture<bool>;
    fn handle(&mut self, msg: IsGameOpen, _: &mut Context<Self>) -> Self::Result {
        let game_addr = self.game_map.lock().unwrap().get(&msg.game_id).unwrap().clone();
        Box::pin(async move {
            game_addr.send(app_to_game::IsGameOpen {}).await.unwrap()
        })
    }
}

/// Handler for NewDirector message.
///
/// Ask the Game to register another director
impl Handler<NewDirector> for AppState {
    type Result = ();
    fn handle(&mut self, msg: NewDirector, _: &mut Context<Self>) -> Self::Result {
        self.game_map.lock().unwrap().get(&msg.game_id).unwrap().do_send(app_to_game::NewDirector {user_id: msg.user_id, username: msg.username});
    }
}

/// Handler for NewGame
/// 
/// Creates a New Game with specified main director
impl Handler<NewGame> for AppState {
    type Result = ();
    fn handle(&mut self, msg: NewGame, context: &mut Context<Self>) -> Self::Result {
        let game = Game::new(context.address(),msg.user_id);
        self.game_map.lock().unwrap().insert(msg.game_id.clone(), game.start());
        println!("Inserted a new game id {}", msg.game_id);
        for key in self.game_map.lock().unwrap().keys() {
            println!("{}", key);
        }
        if let Some(addr) = self.game_map.lock().unwrap().get(&msg.game_id) {
            println!("HI");
            self.temp_addr = Some(addr.clone());
            addr.do_send(app_to_game::IsDirector {user_id: "2".to_string()});
        }
    }
}


/// Handler for IsRegisteredDirector
/// 
/// Creates a New Game with specified main director
impl Handler<ws_to_app::IsRegisteredDirector> for AppState {
    type Result = ResponseFuture<Option<Addr<Game>>>;
    fn handle(&mut self, msg: ws_to_app::IsRegisteredDirector, _context: &mut Context<Self>) -> Self::Result {
        println!("Msg::IsRegisteredDirector");
        let game_map = self.game_map.lock().unwrap();
        for elem in game_map.keys() {
            println!("{}", elem);
        }
        if let Some(addr) = game_map.get(&msg.game_id) {
            let async_addr = addr.clone();
            println!("About to ask game if is a director");
            Box::pin(async move {
               if async_addr.clone().send(app_to_game::IsDirector {user_id: msg.user_id}).await.unwrap() {
                   println!("Told APP that this is a registereddirector ID.");
                   Some(async_addr)
               }
               else {
                   None
               }
            })
        }
        else {
            println!("Could not find with ID {}", msg.game_id);
            Box::pin(async move {
                // async_addr.unwrap().send(app_to_game::IsDirector {user_id: msg.user_id}).await.unwrap();
                None
            })
        }
    }
}

/// Handler for IsPlayer
/// 
/// Creates a New Game with specified main director
impl Handler<ws_to_app::IsPlayer> for AppState {
    type Result = ResponseFuture<Option<Addr<Game>>>;
    fn handle(&mut self, msg: ws_to_app::IsPlayer, _context: &mut Context<Self>) -> Self::Result {
        if let Some(addr) = self.game_map.lock().unwrap().get(&msg.game_id) {
            let async_addr = addr.clone();
            Box::pin(async move {
               if async_addr.clone().send(app_to_game::IsPlayer {user_id: msg.user_id}).await.unwrap() {
                   Some(async_addr)
               }
               else {
                   None
               }
            })
        }
        else {
            Box::pin(async move {None})
        }
    }
}

// impl Handler<IsPlayer> for AppState {
//     type Result = ResponseFuture<Option<Addr<Game>>>;
//     fn handle(&mut self, msg: IsPlayer, _context: &mut Context<Self>) -> Self::Result {
//         if let Some(addr) = self.game_map.lock().unwrap().get(&msg.game_id) {
//             let async_addr = addr.clone();
//             Box::pin(async move {
//                if async_addr.clone().send(app_to_game::IsPlayer {user_id: msg.user_id}).await.unwrap() {
//                    Some(async_addr)
//                }
//                else {
//                    None
//                }
//             })
//         }
//         else {
//             Box::pin(async move {None})
//         }
//     }
// }

