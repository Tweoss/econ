use actix::prelude::*;

/// Notify game of a disconnected participant
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnected {
	pub id: String,
	pub participant_type: String,
}

/// Notify game of an unresponsive participant (not responding to pings)
#[derive(Message)]
#[rtype(result = "()")]
pub struct Unresponsive {
	pub id: String,
	pub participant_type: String,
}

/// Notify game of a responding participant who was previously unresponsive
#[derive(Message)]
#[rtype(result = "()")]
pub struct Responsive {
	pub id: String,
	pub participant_type: String,
}
