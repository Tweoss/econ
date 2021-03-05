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
	pub extra_fields: Option<ServerExtraFields>,
	// pub target: Option<String>,
	// pub info: Option<Info>,
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
	NewOffsets,
	Ignore,
}

#[derive(Debug, Serialize)]
pub struct DirectorClientMsg {
	pub msg_type: DirectorClientType,
	// pub kick_target: Option<String>,
	pub extra_fields: Option<ClientExtraFields>,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum DirectorClientType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
	NewOffsets,
	Pong,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct ServerExtraFields {
	pub target: Option<String>,
	pub info: Option<Info>,
	pub offsets: Option<Offsets>
}


#[derive(Debug, Serialize, Default)]
pub struct ClientExtraFields {
	pub target: Option<String>,
	pub offsets: Option<Offsets>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Offsets {
	pub trending: u8,
	pub supply_shock: u8,
	pub subsidies: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Participant {
	pub state: PlayerState,
	pub took_turn: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum PlayerState {
    Unresponsive,
    Connected,
    Disconnected,
    Kicked,
}
