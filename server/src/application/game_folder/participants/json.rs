use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(11);
pub const CLIENT_TERMINATE: Duration = Duration::from_secs(30);


#[derive(Debug, Serialize)]
pub struct ConsumerServerMsg {
	pub msg_type: ConsumerServerType,
	// If the action requires a target
	pub target: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[allow(dead_code)] 
pub enum ConsumerServerType {
	UnresponsivePlayer,
	GameOpened,
	GameClosed,
	GameEnded,
	ParticipantKicked,
	Ping,
	NewDirector,
	NewConsumer,
	NewProducer,
	NewViewer,
	Ignore,
}

#[derive(Debug, Deserialize)]
pub struct ConsumerClientMsg {
	pub msg_type: ConsumerClientType,
	pub kick_target: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ConsumerClientType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
	Pong,
}


#[derive(Debug, Serialize)]
pub struct ProducerServerMsg {
	pub msg_type: ProducerServerType,
	// If the action requires a target
	pub target: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[allow(dead_code)] 
pub enum ProducerServerType {
	UnresponsivePlayer,
	GameOpened,
	GameClosed,
	GameEnded,
	ParticipantKicked,
	Ping,
	NewDirector,
	NewConsumer,
	NewProducer,
	NewViewer,
	Ignore,
}

#[derive(Debug, Deserialize)]
pub struct ProducerClientMsg {
	pub msg_type: ProducerClientType,
	pub kick_target: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ProducerClientType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
	Pong,
}


#[derive(Debug, Serialize)]
pub struct ViewerServerMsg {
	pub msg_type: ViewerServerType,
	// If the action requires a target
	pub target: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[allow(dead_code)] 
pub enum ViewerServerType {
	UnresponsivePlayer,
	GameOpened,
	GameClosed,
	GameEnded,
	ParticipantKicked,
	Ping,
	NewDirector,
	NewConsumer,
	NewProducer,
	NewViewer,
	Ignore,
}

#[derive(Debug, Deserialize)]
pub struct ViewerClientMsg {
	pub msg_type: ViewerClientType,
	pub kick_target: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ViewerClientType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
	Pong,
}

