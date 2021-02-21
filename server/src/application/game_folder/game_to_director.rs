use actix::prelude::*;

#[derive(Message)]
#[rtype(result="()")]
pub struct Unresponsive {
	pub id: String,
}
