use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ViewerServerMsg {
	pub msg_type: ViewerServerType,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ViewerServerType {
	Info(Info),
	GameEnded,
	GameToggledOpen,
	TurnAdvanced,
	NewOffsets(Offsets),
	NewScores(Vec<(String, f64)>),
	NewParticipant(Participant),
	Ping,
	ServerKicked,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum ViewerClientType {
	Pong,
}

#[derive(Debug, Serialize)]
pub struct ViewerClientMsg {
	pub msg_type: ViewerClientType,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Info {
	pub participants: Vec<Participant>,
	pub turn: u64,
	pub game_id: String,
	pub is_open: bool,
	pub trending: u8,
	pub subsidies: u8,
	pub supply_shock: u8,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Offsets {
	pub trending: u8,
	pub subsidies: u8,
	pub supply_shock: u8,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Participant {
	pub name: String,
	pub is_consumer: bool,
	pub score: f64,
	pub next_index: usize,
}
