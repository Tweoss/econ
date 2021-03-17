use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Info {
	pub info: crate::application::game_folder::participants::director_folder::director_structs::Info,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Unresponsive {
	pub id: String,
	pub participant_type: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnected {
	pub id: String,
	pub participant_type: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connected {
	pub id: String,
	pub participant_type: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewParticipant {
	pub id: String,
	pub participant_type: super::participants::director_folder::director_structs::ParticipantType,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct KickedParticipant {
	pub id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameOpened {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameClosed {}

#[derive(Message)]
#[rtype(result="()")]
pub struct TurnTaken {
	pub id: String,
	pub participant_type: String,
}
