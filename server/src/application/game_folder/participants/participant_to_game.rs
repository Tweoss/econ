use actix::prelude::*;

/// Notify game of a disconnected participant
/// ```
/// #[rtype(result="()")]
/// pub struct Disconnected {
///     pub id: String,
/// }
/// ```
#[derive(Message)]
#[rtype(result="()")]
pub struct Disconnected {
	pub id: String,
}

/// Notify game of an unresponsive participant (not responding to pings)
/// ```
/// #[rtype(result="()")]
/// pub struct Unresponsive {
///     pub id: String,
/// }
/// ```
#[derive(Message)]
#[rtype(result="()")]
pub struct Unresponsive {
	pub id: String,
}