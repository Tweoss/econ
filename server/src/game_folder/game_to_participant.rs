use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct EndedGame {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TurnAdvanced {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Kicked {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewOffsets {
	pub trending: u8,
	pub subsidies: u8,
	pub supply_shock: u8,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StockReduced {
	pub targets: Vec<(String, f64)>,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct Winner {
	pub hash: String,
	pub place: u8,
}
