use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ConsumerServerMsg {
	pub msg_type: ConsumerServerType,
}

#[derive(Debug, Serialize)]
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
}

#[derive(Debug, Deserialize)]
pub enum ConsumerClientType {
	Pong,
	Choice(Vec<(String, f64)>),
	EndTurn,
}

#[derive(Debug, Deserialize)]
pub struct ConsumerClientMsg {
	pub msg_type: ConsumerClientType,
}

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
pub struct TurnInfo {
	pub producers: Vec<(String, Participant)>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Offsets {
	pub trending: u8,
}

// for compatibility with the past_turn game vector
pub use crate::participants::producer_folder::producer_structs::Participant;
