use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ProducerServerMsg {
	pub msg_type: ProducerServerType,
}

#[derive(Debug, Serialize)]
pub enum ProducerServerType {
	Info(Info),
	GameEnded,
	TurnAdvanced(f64, f64),
	TurnInfo(TurnInfo),
	ChoiceSubmitted(f64, f64),
	ChoiceFailed(String),
	NewOffsets(Offsets),
	Ping,
	ServerKicked,
	StockReduced(Vec<(String, f64)>),
	Winner(String, u8),
	GotPurchased(f64),
}
#[derive(Debug, Deserialize)]
pub enum ProducerClientType {
	Pong,
	Choice(ClientExtraFields),
}

#[derive(Debug, Deserialize)]
pub struct ProducerClientMsg {
	pub msg_type: ProducerClientType,
}

#[derive(Debug, Serialize, Clone)]
pub struct Info {
	pub producers: Vec<(String, Participant)>,
	pub turn: u64,
	pub game_id: String,
	pub supply_shock: u8,
	pub subsidies: u8,
	pub balance: f64,
	pub score: f64,
	pub took_turn: bool,
}

#[derive(Debug, Serialize, Clone)]
/// Only sent when producer's turn ends
pub struct TurnInfo {
	pub producers: Vec<(String, Participant)>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Offsets {
	pub supply_shock: u8,
	pub subsidies: u8,
}

#[derive(Debug, Deserialize, Default)]
pub struct ClientExtraFields {
	pub quantity: f64,
	pub price: f64,
	pub t: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct Participant {
	pub produced: f64,
	pub price: f64,
	pub remaining: f64,
}
