use crate::application::game_folder::participants::consumer_folder::consumer::Consumer;
use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterAddressGetInfo {
	pub user_id: String,
	pub addr: Addr<Consumer>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewScoreEndTurn {
	pub user_id: String,
	pub new_score: f64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TryChoice {
	pub user_id: String,
	pub elements: Vec<(String, f64)>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewScoreCalculated {
	pub user_id: String,
	pub new_score: f64,
}
