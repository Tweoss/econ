use actix::prelude::*;

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

/// Close Game
#[derive(Message)]
#[rtype(result="()")]
pub struct CloseGame {}