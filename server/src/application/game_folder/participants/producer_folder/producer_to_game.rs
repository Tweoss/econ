use actix::prelude::*;
use crate::application::game_folder::participants::producer_folder::producer::Producer;

#[derive(Message)]
#[rtype(result="()")]
pub struct RegisterAddressGetInfo {
	pub user_id: String,
	pub addr: Addr<Producer>,
}