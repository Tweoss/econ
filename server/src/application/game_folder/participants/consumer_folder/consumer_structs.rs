use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct ConsumerServerMsg {
	pub msg_type: ConsumerServerType,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum ConsumerServerType {
	Info(Info),
	GameEnded,
	TurnAdvanced((f64, f64, f64)),
	TurnInfo(TurnInfo),
	TurnEnded,
	ChoiceSubmitted((f64, f64, f64)),
	ChoiceFailed(String),
	NewOffsets(Offsets),
	Ping,
	ServerKicked,
	StockReduced(Vec<(String, f64)>),
	Ignore,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ConsumerClientType {
	Pong,	
	Choice,
	EndTurn,
}

#[derive(Debug, Deserialize)]
pub struct ConsumerClientMsg {
	pub msg_type: ConsumerClientType,
	pub choice: Option<ClientExtraFields>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct TurnInfo {
	pub producers: Vec<(String, Participant)>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Offsets {
	pub trending: u8,
}

#[derive(Debug, Deserialize, Default)]
pub struct ClientExtraFields {
	pub elements: Vec<(String, f64)>,
}

// for compatibility with the past_turn game vector
pub use crate::application::game_folder::participants::producer_folder::producer_structs::Participant;