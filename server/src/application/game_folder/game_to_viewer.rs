use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Info {
	pub info:
		crate::application::game_folder::participants::viewer_folder::viewer_structs::Info,
}