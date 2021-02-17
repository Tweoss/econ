use actix::prelude::*;
use crate::application::game::Game;

/// See if a director with this ID was previously authenticated and if so, return the correct game address
/// ```
/// #[rtype(result="Option<Addr<Game>>")]
/// pub struct IsRegisteredDirector {
///     pub user_id: String,
///     pub game_id: String,
/// }
/// ```
#[derive(Message)]
#[rtype(result="Option<Addr<Game>>")]
pub struct IsRegisteredDirector {
    pub user_id: String,
    pub game_id: String,
}

/// See if a player with this ID was previously registered and if so, return the correct game address
/// ```
/// #[rtype(result="Option<Addr<Game>>")]
/// pub struct IsPlayer {
///     pub user_id: String,
///     pub game_id: String,
/// }
/// ```
#[derive(Message)]
#[rtype(result="Option<Addr<Game>>")]
pub struct IsPlayer {
    pub user_id: String,
    pub game_id: String,
}