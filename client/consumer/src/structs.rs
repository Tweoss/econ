use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ConsumerServerMsg {
	pub msg_type: ConsumerServerType,
	pub extra_fields: Option<ServerExtraFields>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ConsumerServerType {
	Info(Info),
	GameEnded,
	TurnAdvanced,
	TurnInfo,
	ChoiceSubmitted,
	ChoiceFailed,
	NewOffsets,
	Ping,
	ServerKicked,
	StockReduced,
	Ignore,
}
#[derive(Debug, Serialize, PartialEq)]
pub enum ConsumerClientType {
	Pong,	
	// Choice(Vec<(String, f64)>),
	Choice,
	EndTurn,
}

#[derive(Debug, Serialize)]
pub struct ConsumerClientMsg {
	pub msg_type: ConsumerClientType,
	pub choice: Option<ClientExtraFields>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct TurnInfo {
	pub producers: Vec<(String, Participant)>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ServerExtraFields {
	// pub info: Option<Info>,
	pub offsets: Option<Offsets>,
	pub turn_info: Option<TurnInfo>,
	pub fail_info: Option<String>,
	// * Remaining 
	pub purchased: Option<(String, f64)>,
	// * New Score after Turn ends (moves from balance to score)
	pub balance_score_quantity: Option<(f64, f64, f64)>,
	pub stock_targets: Option<Vec<(String, f64)>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Offsets {
	pub trending: u8,
}

#[derive(Debug, Serialize, Default)]
pub struct ClientExtraFields {
	pub elements: Vec<(String, f64)>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Participant {
	pub produced: f64,
	pub price: f64, 
	pub remaining: f64,
}