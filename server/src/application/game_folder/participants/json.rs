use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ParticipantType {
	Director,
	Producer,
	Consumer,
	Viewer,
}

#[derive(Debug, Serialize)]
pub struct DirectorServerMsg {
	pub msg_type: DirectorServerType,
	// If the action requires a target
	pub target: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[allow(dead_code)] 
pub enum DirectorServerType {
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
pub struct DirectorClientMsg {
	pub msg_type: DirectorClientType,
	pub kick_target: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum DirectorClientType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
	Pong,
}


#[derive(Debug, Serialize)]
pub struct ConsumerServerMsg {
	pub msg_type: DirectorServerType,
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
	pub msg_type: DirectorClientType,
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
	pub msg_type: DirectorServerType,
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
	pub msg_type: DirectorClientType,
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
	pub msg_type: DirectorServerType,
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
	pub msg_type: DirectorClientType,
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

