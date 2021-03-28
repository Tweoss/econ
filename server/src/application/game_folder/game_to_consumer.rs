use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Info {
	pub info:
		crate::application::game_folder::participants::consumer_folder::consumer_structs::Info,
}