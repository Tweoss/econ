use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Info {
	pub info: crate::application::game_folder::participants::viewer_folder::viewer_structs::Info,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameOpened {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameClosed {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewScores {
	pub list: Vec<(String, f64)>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewParticipant {
	pub name: String,
	pub is_consumer: bool,
}
