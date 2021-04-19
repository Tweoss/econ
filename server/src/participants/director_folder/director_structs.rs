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
}

#[derive(Debug, Serialize)]
pub enum DirectorServerType {
	Info(Info),
	UnresponsivePlayer(String, String),
	DisconnectedPlayer(String, String),
	ConnectedPlayer(String, String),
	GameOpened,
	GameClosed,
	GameEnded,
	TurnAdvanced,
	ParticipantKicked(String),
	TurnTaken(String, String),
	Ping,
	ServerKicked,
	NewDirector(String, String),
	NewConsumer(String, String),
	NewProducer(String, String),
	NewViewer(String, String),
	NewOffsets(Offsets),
	Winners([Option<Vec<(String, String)>>; 3]),
}

#[derive(Debug, Deserialize)]
pub struct DirectorClientMsg {
	pub msg_type: DirectorClientType,
}

#[derive(Debug, Deserialize)]
pub enum DirectorClientType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick(String),
	NewOffsets(Offsets),
	Pong,
	NextTurn,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Offsets {
	pub trending: u8,
	pub supply_shock: u8,
	pub subsidies: u8,
}

#[derive(Debug, Serialize)]
pub struct Participant {
	pub state: PlayerState,
	pub took_turn: Option<bool>,
	pub id: String,
}

#[derive(Debug, Serialize)]
pub enum PlayerState {
	Unresponsive,
	Connected,
	Disconnected,
}
