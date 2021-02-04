// MESSAGES FROM HANDLERS TO THE APPLICATION

use actix::prelude::*;

/// Check if a game exists
#[derive(Message)]
#[rtype(bool)]
pub struct DoesGameExist {
	pub game_id: String,
}
/// Check if a game exists
#[derive(Message)]
#[rtype(bool)]
pub struct IsGameOpen {
	pub game_id: String,
}
/// Check if a game exists
#[derive(Message)]
#[rtype(bool)]
pub struct IsRightPswd {
	pub pswd: String,
}

/// Register a new player's ID in a game
#[derive(Message)]
#[rtype(String)]
pub struct NewPlayer {
    pub user_id: String,
    pub game_id: String,
    pub username: String,
}

/// Register ANOTHER director in a game
#[derive(Message)]
#[rtype(result="()")]
pub struct NewDirector {
    pub user_id: String,
    pub game_id: String,
    pub username: String,
}

/// Create a new game
#[derive(Message)]
#[rtype(result="()")]
pub struct NewGame {
    pub user_id: String,
    pub game_id: String,
    pub username: String,
}