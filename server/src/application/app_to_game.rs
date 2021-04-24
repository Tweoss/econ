use actix::prelude::*;

/// Register a new player \
/// Forwarded from App
#[derive(Message)]
#[rtype(String)]
pub struct NewPlayer {
	pub user_id: String,
	pub username: String,
}

/// Registering a new director
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewDirector {
	pub user_id: String,
	pub username: String,
}

/// Registering a new director
#[derive(Message)]
#[rtype(bool)]
pub struct NewViewer {
	pub user_id: String,
	pub username: String,
}

/// Check if Game is open to players
#[derive(Message)]
#[rtype(bool)]
pub struct IsGameOpen {}

#[derive(Message)]
#[rtype(result = "Option<String>")]
pub struct IsDirector {
	pub user_id: String,
}

#[derive(Message)]
#[rtype(result = "Option<String>")]
pub struct IsPlayer {
	pub user_id: String,
}

#[derive(Message)]
#[rtype(result = "Option<String>")]
pub struct IsViewer {
	pub user_id: String,
}

/// Check if this id is the main director
#[derive(Message)]
#[rtype(bool)]
pub struct IsMainDirector {
	pub user_id: String,
}
