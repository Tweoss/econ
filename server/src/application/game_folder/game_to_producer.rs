use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Info {
	pub info:
		crate::application::game_folder::participants::producer_folder::producer_structs::Info,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TurnList {
	pub list: Vec<(String, crate::application::game_folder::participants::producer_folder::producer_structs::Participant)>,
}
