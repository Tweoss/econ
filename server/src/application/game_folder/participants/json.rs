use serde::{Deserialize, Serialize};

#[derive(Debug,Deserialize,Serialize)]
pub struct DirectorData {
	pub msg_type: DirectorMsgType,
	pub kick_target: Option<String>,
}

#[derive(Debug,Deserialize,Serialize)]
pub enum DirectorMsgType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
}