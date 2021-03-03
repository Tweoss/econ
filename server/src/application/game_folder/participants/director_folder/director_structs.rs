use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ParticipantType {
	Director,
	Producer,
	Consumer,
	Viewer,
}

#[derive(Debug, Serialize)]
pub struct DirectorServerMsg {
	pub msg_type: DirectorServerType,
	// If the action requires a target
	pub target: Option<String>,
	pub info: Option<Info>,
}

#[derive(Debug, Serialize, PartialEq)]

#[allow(dead_code)] 
pub enum DirectorServerType {
	Info,
	UnresponsivePlayer,
	GameOpened,
	GameClosed,
	GameEnded,
	ParticipantKicked,
	Ping,
	NewDirector,
	NewConsumer,
	NewProducer,
	NewViewer,
	Ignore,
}

#[derive(Debug, Deserialize)]
pub struct DirectorClientMsg {
	pub msg_type: DirectorClientType,
	pub kick_target: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum DirectorClientType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
	Pong,
}

#[derive(Debug, Serialize)]
pub struct Info {
	pub consumers: Vec<(String, Participant)>,
	pub producers: Vec<(String, Participant)>,
	pub directors: Vec<(String, Participant)>,
	pub viewers: Vec<(String, Participant)>,
	pub is_open: bool,
	pub turn: u64,
	pub game_id: String,
}

#[derive(Debug, Serialize)]
pub struct Participant {
	pub state: PlayerState,
	pub took_turn: Option<bool>,
}

#[derive(Debug, Serialize)]
pub enum PlayerState {
    Unresponsive,
    Connected,
    Disconnected,
    Kicked,
}
