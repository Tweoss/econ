use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ConsumerServerMsg {
	pub msg_type: ConsumerServerType,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ConsumerServerType {
	Info(Info),
	GameEnded,
	TurnAdvanced(f64, f64),
	TurnInfo(TurnInfo),
	TurnEnded,
	ChoiceSubmitted(f64, f64, f64),
	NewOffsets(Offsets),
	Ping,
	ServerKicked,
	StockReduced(Vec<(String, f64)>),
	Winner(String, u8),
}

#[derive(Debug, Serialize, PartialEq)]
pub enum ConsumerClientType {
	Pong,	
	Choice(Vec<(String, f64)>),
	EndTurn,
}

#[derive(Debug, Serialize)]
pub struct ConsumerClientMsg {
	pub msg_type: ConsumerClientType,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Info {
	pub producers: Vec<(String, Participant)>,
	pub turn: u64,
	pub game_id: String,
	pub trending: u8,
	pub balance: f64,
	pub quantity_purchased: f64,
	pub score: f64,
	pub took_turn: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TurnInfo {
	pub producers: Vec<(String, Participant)>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Offsets {
	pub trending: u8,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Participant {
	pub produced: f64,
	pub price: f64, 
	pub remaining: f64,
}