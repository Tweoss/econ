use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Info {
	pub info:
		crate::application::game_folder::participants::consumer_folder::consumer_structs::Info,
}

#[derive(Message)]
#[rtype(result="()")]
pub struct PurchaseResult {
	pub expense: f64,
	pub balance: f64,
	pub purchased: f64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TurnList {
	pub list: Vec<(String, crate::application::game_folder::participants::producer_folder::producer_structs::Participant)>,
}
