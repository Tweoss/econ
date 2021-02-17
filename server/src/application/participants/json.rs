pub struct DirectorJson {
	msg_type: DirectorMsgType,
	kick_target: Option<String>,
}

pub enum DirectorMsgType {
	OpenGame,
	CloseGame,
	EndGame,
	Kick,
}