//* list of messages that can be sent to Actors (App, Game, Player, Viewer, Director)

use actix::prelude::*;

/// Check if a game exists
#[derive(Message)]
#[rtype(bool)]
pub struct DoesGameExist {
	pub game_id: String,
}

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewViewer(pub String); 


/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}