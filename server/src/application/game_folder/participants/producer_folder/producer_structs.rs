use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ProducerServerMsg {
	pub msg_type: ProducerServerType,
	pub extra_fields: Option<ServerExtraFields>,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum ProducerServerType {
	Info,
	GameEnded,
	TurnAdvanced,
	ChoiceSubmitted,
	ChoiceFailed,
	QuantityPurchased,
	Ping,
	ServerKicked,
	Ignore,
}
#[derive(Debug, Deserialize, PartialEq)]
pub enum ProducerClientType {
	Pong,	
	Choice,
}

#[derive(Debug, Deserialize)]
pub struct ProducerClientMsg {
	pub msg_type: ProducerClientType,
	pub choice: Option<ClientExtraFields>,
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
}

#[derive(Debug, Serialize, Clone)]
pub struct TurnInfo {
	pub producers: Vec<(String, Participant)>,
	pub offsets: Offsets,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct ServerExtraFields {
	pub info: Option<Info>,
	pub offsets: Option<Offsets>,
	pub turn_info: Option<TurnInfo>,
	// * Remaining balance
	pub submitted_info: Option<f64>,
	pub fail_info: Option<String>,
	// * New score
	pub purchased: Option<f64>,
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
}

#[derive(Debug, Serialize, Clone)]
pub struct Participant {
	pub produced: f64,
	pub price: f64, 
	pub remaining: f64,
}