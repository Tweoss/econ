use serde::{Deserialize, Serialize};

// #[derive(Debug)]
// pub enum ParticipantType {
// 	Director,
// 	Producer,
// 	Consumer,
// 	Viewer,
// }

#[derive(Debug, Deserialize)]
pub struct ProducerServerMsg {
	pub msg_type: ProducerServerType,
	pub extra_fields: Option<ServerExtraFields>,
}

#[derive(Debug, Deserialize, PartialEq)]
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
#[derive(Debug, Serialize, PartialEq)]
pub enum ProducerClientType {
	Pong,	
	Choice,
}

#[derive(Debug, Serialize)]
pub struct ProducerClientMsg {
	pub msg_type: ProducerClientType,
	pub choice: Option<ClientExtraFields>,
}

#[derive(Debug, Deserialize, Clone)]
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
	pub offsets: Offsets,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerExtraFields {
	pub info: Option<Info>,
	pub offsets: Option<Offsets>,
	pub turn_info: Option<TurnInfo>,
	// * Remaining balance
	//* score and balance
	pub submitted_info: Option<(f64, f64)>,
	pub fail_info: Option<String>,
	// * New score
	pub purchased: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Offsets {
	pub supply_shock: u8,
	pub subsidies: u8,
}

#[derive(Debug, Serialize, Default)]
pub struct ClientExtraFields {
	pub quantity: f64,
	pub price: f64,
	pub t: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Participant {
	pub produced: f64,
	pub price: f64, 
	pub remaining: f64,
}