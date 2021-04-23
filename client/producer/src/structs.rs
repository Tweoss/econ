use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ProducerServerMsg {
	pub msg_type: ProducerServerType,
}

#[derive(Debug, Deserialize)]
pub enum ProducerServerType {
	Info(Info),
	GameEnded,
	TurnAdvanced(f64, f64),
	TurnInfo(TurnInfo),
	ChoiceSubmitted(f64,f64),
	ChoiceFailed(String),
	NewOffsets(Offsets),
	Ping,
	ServerKicked,
	StockReduced(Vec<(String, f64)>),
	GotPurchased(f64),
	Winner(String, u8),
}

#[derive(Debug, Serialize, PartialEq)]
pub enum ProducerClientType {
	Pong,	
	Choice(ClientExtraFields),
}

#[derive(Debug, Serialize)]
pub struct ProducerClientMsg {
	pub msg_type: ProducerClientType,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct TurnInfo {
	pub producers: Vec<(String, Participant)>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Offsets {
	pub supply_shock: u8,
	pub subsidies: u8,
}

#[derive(Debug, Serialize, Default, PartialEq)]
pub struct ClientExtraFields {
	pub quantity: f64,
	pub price: f64,
	pub t: f64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Participant {
	pub produced: f64,
	pub price: f64, 
	pub remaining: f64,
}