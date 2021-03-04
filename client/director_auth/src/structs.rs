use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ParticipantType {
	Director,
	Producer,
	Consumer,
	Viewer,
}

#[derive(Debug, Deserialize)]
pub struct DirectorServerMsg {
	pub msg_type: DirectorServerType,
	// If the action requires a target
	pub target: Option<String>,
	pub info: Option<Info>,
}

#[derive(Debug, Deserialize, PartialEq)]
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

#[derive(Debug, Serialize)]
pub struct DirectorClientMsg {
	pub msg_type: DirectorClientType,
	pub kick_target: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum DirectorClientType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
	Pong,
}

#[derive(Debug, Deserialize)]
pub struct Info {
	pub consumers: Vec<(String, Participant)>,
	pub producers: Vec<(String, Participant)>,
	pub directors: Vec<(String, Participant)>,
	pub viewers: Vec<(String, Participant)>,
	pub is_open: bool,
	pub turn: u64,
	pub trending: u8,
	pub supply_shock: u8,
	pub subsidies: u8,
	pub game_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Participant {
	pub state: PlayerState,
	pub took_turn: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub enum PlayerState {
    Unresponsive,
    Connected,
    Disconnected,
    Kicked,
}
