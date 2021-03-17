use actix::prelude::*;
use crate::application::game_folder::participants::producer_folder::producer::Producer;

#[derive(Message)]
#[rtype(result="()")]
pub struct RegisterAddressGetInfo {
	pub user_id: String,
	pub addr: Addr<Producer>,
}

#[derive(Message)]
#[rtype(result="()")]
pub struct NewScoreEndTurn {
	pub user_id: String,
	pub new_score: f64,
}