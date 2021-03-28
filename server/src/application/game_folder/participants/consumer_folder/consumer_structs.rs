use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct ConsumerServerMsg {
	pub msg_type: ConsumerServerType,
	pub extra_fields: Option<ServerExtraFields>,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum ConsumerServerType {
	Info,
	GameEnded,
	TurnAdvanced,
	TurnInfo,
	ChoiceSubmitted,
	ChoiceFailed,
	QuantityPurchased,
	NewOffsets,
	Ping,
	ServerKicked,
	Ignore,
}
#[derive(Debug, Deserialize, PartialEq)]
pub enum ConsumerClientType {
	Pong,	
	Choice,
}

#[derive(Debug, Deserialize)]
pub struct ConsumerClientMsg {
	pub msg_type: ConsumerClientType,
	pub choice: Option<ClientExtraFields>,
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

#[derive(Debug, Serialize, Clone, Default)]
pub struct ServerExtraFields {
	pub info: Option<Info>,
	pub offsets: Option<Offsets>,
	pub turn_info: Option<TurnInfo>,
	// * Remaining balance
	//* score and balance
	pub submitted_info: Option<(f64, f64)>,
	pub fail_info: Option<String>,
	// * Remaining 
	pub purchased: Option<(String, f64)>,
	// * New Score after Turn ends (moves from balance to score)
	pub score: Option<f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Offsets {
	pub trending: u8,
}

#[derive(Debug, Deserialize, Default)]
pub struct ClientExtraFields {
	pub elements: Vec<(String, f64)>,
	pub t: f64,
}

// for compatibility with the past_turn game vector
pub use crate::application::game_folder::participants::producer_folder::producer_structs::Participant;