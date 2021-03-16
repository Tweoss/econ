use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Info {
	pub info: crate::application::game_folder::participants::producer_folder::producer_structs::Info,
}
