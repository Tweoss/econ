use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Info {
	pub info:
		crate::application::game_folder::participants::director_folder::director_structs::Info,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Unresponsive {
	pub name: String,
	pub participant_type: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnected {
	pub name: String,
	pub participant_type: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connected {
	pub name: String,
	pub participant_type: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewParticipant {
	pub id: String,
	pub name: String,
	pub participant_type: super::participants::director_folder::director_structs::ParticipantType,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct KickedParticipant {
	pub name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameOpened {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameClosed {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TurnTaken {
	pub name: String,
	pub participant_type: String,
}
