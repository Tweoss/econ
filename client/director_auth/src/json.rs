use serde::{Deserialize, Serialize};

#[derive(Debug,Deserialize,Serialize)]
pub struct DirectorClientMsg {
	pub msg_type: DirectorClientType,
	pub kick_target: Option<String>,
}

#[derive(Debug,Deserialize,Serialize)]
pub enum DirectorClientType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
}

#[derive(Debug,Deserialize)]
pub struct DirectorServerMsg {
	pub msg_type: DirectorServerType,
	// If the action requires a target
	pub target: Option<String>,
}

#[derive(Debug,Deserialize)]
pub enum DirectorServerType {
	UnresponsivePlayer,
	GameEnded,
	Ignore,
}