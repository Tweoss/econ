use actix::prelude::*;
use crate::application::game_folder::participants::consumer_folder::consumer::Consumer;

#[derive(Message)]
#[rtype(result="()")]
pub struct RegisterAddressGetInfo {
	pub user_id: String,
	pub addr: Addr<Consumer>,
}
