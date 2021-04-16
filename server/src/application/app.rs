use actix::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use crate::application::app_to_game;
use crate::game_folder::game::Game;
use crate::game_folder::game_to_app;
use crate::handle_to_app::*;

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
    game_map: RwLock<HashMap<String, Addr<Game>>>,
    // game_ids: Mutex<Vec<String>>,
    password_hash: u64,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            game_map: RwLock::new(HashMap::new()),
            // game_ids: Mutex::new(Vec::new()),
            password_hash: 16528679900032520146,
            // password_hash: 9612577385432581406,
        }
    }
}

impl Actor for AppState {
    type Context = actix::Context<Self>;
}

/// Handler for DoesGameExist message.
///
/// Check if a game_id is valid
impl Handler<DoesGameExist> for AppState {
    type Result = bool;
    fn handle(&mut self, msg: DoesGameExist, _: &mut Context<Self>) -> Self::Result {
        println!("msg: DoesGameExist");
        let string = msg.game_id;
        // self.game_map.read().unwrap().iter().any(|x| x.0 == string);
        self.game_map.read().unwrap().contains_key(&string)
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
        msg.pswd.hash(&mut hasher);
        hasher.finish() == self.password_hash
    }
}

/// Handler for NewPlayer message.
///
/// Ask the Game to register a new player
impl Handler<NewPlayer> for AppState {
    type Result = ResponseFuture<String>;
    fn handle(&mut self, msg: NewPlayer, _: &mut Context<Self>) -> Self::Result {
        // let game_addr = self.game_map.read().unwrap().iter().find(|&x| x.0 == msg.game_id).unwrap().1.clone();
        let game_addr = self
            .game_map
            .read()
            .unwrap()
            .get(&msg.game_id)
            .unwrap()
            .clone();
        Box::pin(async move {
            game_addr
                .send(app_to_game::NewPlayer {
                    user_id: msg.user_id,
                    username: msg.username,
                })
                .await
                .unwrap()
        })
    }
}

impl Handler<NewViewer> for AppState {
    type Result = ResponseFuture<bool>;
    fn handle(&mut self, msg: NewViewer, _: &mut Context<Self>) -> Self::Result {
        // let game_addr = self.game_map.read().unwrap().iter().find(|&x| x.0 == msg.game_id).unwrap().1.clone();
        let game_addr = self
            .game_map
            .read()
            .unwrap()
            .get(&msg.game_id)
            .unwrap()
            .clone();
        Box::pin(async move {
            game_addr
                .send(app_to_game::NewViewer {
                    user_id: msg.user_id,
                    username: msg.username,
                })
                .await
                .unwrap()
        })
    }
}

impl Handler<IsGameOpen> for AppState {
    type Result = ResponseFuture<bool>;
    fn handle(&mut self, msg: IsGameOpen, _: &mut Context<Self>) -> Self::Result {
        // let game_addr = self.game_map.read().unwrap().iter().find(|&x| x.0 == msg.game_id).unwrap().1.clone();
        let game_addr = self
            .game_map
            .read()
            .unwrap()
            .get(&msg.game_id)
            .unwrap()
            .clone();
        Box::pin(async move { game_addr.send(app_to_game::IsGameOpen {}).await.unwrap() })
    }
}

/// Handler for NewDirector message.
///
/// Ask the Game to register another director
impl Handler<NewDirector> for AppState {
    type Result = ();
    fn handle(&mut self, msg: NewDirector, _: &mut Context<Self>) -> Self::Result {
        self.game_map
            .read()
            .unwrap()
            .get(&msg.game_id)
            .unwrap()
            // .iter()
            // .find(|&x| x.0 == msg.game_id)
            // .unwrap()
            // .1
            .do_send(app_to_game::NewDirector {
                user_id: msg.user_id,
                username: msg.username,
            });
    }
}

/// Handler for NewGame
///
/// Creates a New Game with specified main director
impl Handler<NewGame> for AppState {
    type Result = ();
    fn handle(&mut self, msg: NewGame, context: &mut Context<Self>) -> Self::Result {
        let game = Game::new(
            context.address(),
            msg.user_id,
            msg.username,
            msg.game_id.clone(),
        );
        self.game_map
            .write()
            .unwrap()
            .insert(msg.game_id.clone(), game.start());
        // .insert(0, (msg.game_id.clone(), game.start()));
        println!("Inserted a new game id {}", msg.game_id);
    }
}

impl Handler<IsMainDirector> for AppState {
    type Result = ResponseFuture<bool>;
    fn handle(&mut self, msg: IsMainDirector, _context: &mut Context<Self>) -> Self::Result {
        if let Some(game_addr) = self.game_map.read().unwrap().get(&msg.game_id) {
            let game_addr_clone = game_addr.clone();
            Box::pin(async move {
                game_addr_clone
                    .send(app_to_game::IsMainDirector {
                        user_id: msg.user_id,
                    })
                    .await
                    .unwrap()
            })
        } else {
            Box::pin(async move { false })
        }
    }
}

/// Handler for IsRegisteredDirector
impl Handler<IsRegisteredDirector> for AppState {
    type Result = ResponseFuture<Option<(Addr<Game>, String)>>;
    fn handle(&mut self, msg: IsRegisteredDirector, _context: &mut Context<Self>) -> Self::Result {
        println!("Msg::IsRegisteredDirector");
        if let Some(addr) = self.game_map.read().unwrap().get(&msg.game_id)
        // .iter()
        // .find(|&x| x.0 == msg.game_id)
        {
            let async_addr = addr.clone();
            // let async_addr = addr.1.clone();
            Box::pin(async move {
                if let Some(name) = async_addr
                    .clone()
                    .send(app_to_game::IsDirector {
                        user_id: msg.user_id,
                    })
                    .await
                    .unwrap()
                {
                    Some((async_addr, name))
                } else {
                    None
                }
            })
        } else {
            println!("Could not find with ID {}", msg.game_id);
            Box::pin(async move { None })
        }
    }
}

impl Handler<IsRegisteredViewer> for AppState {
    type Result = ResponseFuture<Option<(Addr<Game>, String)>>;
    fn handle(&mut self, msg: IsRegisteredViewer, _context: &mut Context<Self>) -> Self::Result {
        if let Some(addr) = self.game_map.read().unwrap().get(&msg.game_id) {
            let async_addr = addr.clone();
            Box::pin(async move {
                if let Some(name) = async_addr
                    .clone()
                    .send(app_to_game::IsViewer {
                        user_id: msg.user_id,
                    })
                    .await
                    .unwrap()
                {
                    Some((async_addr, name))
                } else {
                    None
                }
            })
        } else {
            Box::pin(async move { None })
        }
    }
}

impl Handler<IsRegisteredPlayer> for AppState {
    type Result = ResponseFuture<Option<(Addr<Game>, String)>>;
    fn handle(&mut self, msg: IsRegisteredPlayer, _context: &mut Context<Self>) -> Self::Result {
        if let Some(addr) = self.game_map.read().unwrap().get(&msg.game_id) {
            let async_addr = addr.clone();
            Box::pin(async move {
                if let Some(name) = async_addr
                    .clone()
                    .send(app_to_game::IsPlayer {
                        user_id: msg.user_id,
                    })
                    .await
                    .unwrap()
                {
                    Some((async_addr, name))
                } else {
                    None
                }
            })
        } else {
            println!("Could not find game with ID {}", msg.game_id);
            Box::pin(async move { None })
        }
    }
}

impl Handler<game_to_app::EndGame> for AppState {
    type Result = ();
    fn handle(&mut self, msg: game_to_app::EndGame, _: &mut Context<Self>) -> Self::Result {
        // let mut vec = self.game_map.write().unwrap();
        // let index = vec.iter().position(|elem| elem.0 == msg.game_id).unwrap();
        // vec.remove(index);
        self.game_map.write().unwrap().remove(&msg.game_id);
        println!("Removed a game from app");
    }
}
