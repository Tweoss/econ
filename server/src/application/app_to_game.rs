use actix::prelude::*;
/// Register a new player \
/// Forwarded from App
/// ```
/// pub struct NewPlayer {
///     pub user_id: String,
///     pub username: String,
/// }
/// ```
#[derive(Message)]
#[rtype(String)]
pub struct NewPlayer {
	pub user_id: String,
	pub username: String,
}

/// Registering a new director
#[derive(Message)]
#[rtype(result="()")]
pub struct NewDirector {
	pub user_id: String,
	pub username: String,
}

/// Check if Game is open to players
#[derive(Message)]
#[rtype(bool)]
pub struct IsGameOpen {}