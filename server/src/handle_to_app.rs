// MESSAGES FROM HANDLERS TO THE APPLICATION

use actix::prelude::*;
// use crate::application::app::AppState;
use crate::application::game_folder::game::Game;

/// Check if a game exists
/// ```
/// #[rtype(bool)]
/// pub struct DoesGameExist {
///     pub game_id: String,
/// }
/// ```
#[derive(Message)]
#[rtype(bool)]
pub struct DoesGameExist {
    pub game_id: String,
}

/// Check if a game is open to players joining
/// ```
/// #[rtype(bool)]
/// pub struct IsGameOpen {
///     pub game_id: String,
/// }
/// ```
#[derive(Message)]
#[rtype(bool)]
pub struct IsGameOpen {
    pub game_id: String,
}

/// Check if this is the Right Director Password
/// ```
/// #[rtype(bool)]
/// pub struct IsRightPswd {
///     pub pswd: String,
/// }
/// ```
#[derive(Message)]
#[rtype(bool)]
pub struct IsRightPswd {
    pub pswd: String,
}

/// Register a new player's ID in a game
/// ```
/// #[rtype(String)]
/// pub struct NewPlayer {
///     pub user_id: String,
///     pub game_id: String,
///     pub username: String,
/// }
/// ```
#[derive(Message)]
#[rtype(String)]
pub struct NewPlayer {
    pub user_id: String,
    pub game_id: String,
    pub username: String,
}

/// Register ANOTHER director in a game
/// ```
/// #[rtype(result="()")]
/// pub struct NewDirector {
///     pub user_id: String,
///     pub game_id: String,
///     pub username: String,
/// }
/// ```
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewDirector {
    pub user_id: String,
    pub game_id: String,
    pub username: String,
}

#[derive(Message)]
#[rtype(bool)]
pub struct NewViewer {
    pub user_id: String,
    pub game_id: String,
    pub username: String,
}

/// Create a new game
/// ```
/// #[rtype(result="()")]
/// pub struct NewGame {
///     pub user_id: String,
///     pub game_id: String,
///     pub username: String,
/// }
/// ```
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewGame {
    pub user_id: String,
    pub username: String,
    pub game_id: String,
}

/// Prevent main directors from joining other games
/// ```
/// #[rtype(result="()")]
/// pub struct NewGame {
///     pub user_id: String,
///     pub game_id: String,
///     pub username: String,
/// }
/// ```
#[derive(Message)]
#[rtype(bool)]
pub struct IsMainDirector {
    pub game_id: String,
    pub user_id: String,
}

/// See if a director with this ID was previously authenticated and if so, return the correct game address
#[derive(Message)]
#[rtype(result = "Option<Addr<Game>>")]
pub struct IsRegisteredDirector {
    pub user_id: String,
    pub game_id: String,
}

/// See if producer or consumer with this ID was previously authenticated and if so, return the correct game address
#[derive(Message)]
#[rtype(result = "Option<Addr<Game>>")]
pub struct IsRegisteredPlayer {
    pub user_id: String,
    pub game_id: String,
}

#[derive(Message)]
#[rtype(result = "Option<Addr<Game>>")]
pub struct IsRegisteredViewer {
    pub user_id: String,
    pub game_id: String,
}
