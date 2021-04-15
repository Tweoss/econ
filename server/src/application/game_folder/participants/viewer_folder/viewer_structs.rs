use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct ViewerServerMsg {
	pub msg_type: ViewerServerType,
}

#[derive(Debug, Serialize)]
pub enum ViewerServerType {
	Info(Info),
	GameEnded,
	GameOpened,
	GameClosed,
	TurnAdvanced,
	NewOffsets(Offsets),
	NewScores(Vec<(String, f64)>),
	NewParticipant(Participant),
	Ping,
	ServerKicked,
}

#[derive(Debug, Deserialize)]
pub enum ViewerClientType {
	Pong,
}

#[derive(Debug, Deserialize)]
pub struct ViewerClientMsg {
	pub msg_type: ViewerClientType,
}

#[derive(Debug, Serialize, Clone)]
pub struct Info {
	pub participants: Vec<Participant>,
	pub turn: u64,
	pub game_id: String,
	pub is_open: bool,
	pub trending: u8,
	pub subsidies: u8,
	pub supply_shock: u8,
}

#[derive(Debug, Serialize, Clone)]
pub struct Offsets {
	pub trending: u8,
	pub subsidies: u8,
	pub supply_shock: u8,
}

#[derive(Debug, Serialize, Clone)]
pub struct Participant {
	pub name: String,
	pub is_consumer: bool,
	pub score: f64,
	pub next_index: usize,
}
