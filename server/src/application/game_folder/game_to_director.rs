use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Unresponsive {
	pub id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewParticipant {
	pub id: String,
	pub participant_type: super::participants::director_folder::director_structs::ParticipantType,
}

// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct NewProducer {
// 	pub id: String,
// }

// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct NewViewer {
// 	pub id: String,
// }

// #[derive(Message)]
// #[rtype(result = "()")]
// pub struct NewDirector {
// 	pub id: String,
// }

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
