use actix::prelude::*;
use crate::application::game_folder::participants::director_folder::director::Director;

#[derive(Message)]
#[rtype(result="()")]
pub struct RegisterAddressGetInfo {
	pub user_id: String,
	pub addr: Addr<Director>,
}

/// Check if this consumer is registered
/// ```
/// #[rtype(bool)]
/// pub struct IsConsumer {
///     pub user_id: String,
/// }
/// ```
#[derive(Message)]
#[rtype(bool)]
pub struct IsConsumer {
	pub user_id: String,
}

/// Check if this producer is registered
#[derive(Message)]
#[rtype(bool)]
pub struct IsProducer {
	pub user_id: String,
}

/// Check if this viewer is registered
#[derive(Message)]
#[rtype(bool)]
pub struct IsViewer {
	pub user_id: String,
}

/// Try to Kick a player. returns if succeeded
#[derive(Message)]
#[rtype(result="()")]
pub struct KickParticipant {
	pub user_id: String,
}

/// End Game
#[derive(Message)]
#[rtype(result="()")]
pub struct EndGame {}

/// Open Game
#[derive(Message)]
#[rtype(result="()")]
pub struct OpenGame {
}

/// Close Game
#[derive(Message)]
#[rtype(result="()")]
pub struct CloseGame {
}