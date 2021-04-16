use actix::prelude::*;
use crate::participants::producer_folder::producer::Producer;

#[derive(Message)]
#[rtype(result="()")]
pub struct RegisterAddressGetInfo {
	pub name: String,
	pub addr: Addr<Producer>,
}

#[derive(Message)]
#[rtype(result="()")]
pub struct NewScoreEndTurn {
	pub name: String,
	pub new_score: f64,
	pub produced: f64,
	pub price: f64,
}