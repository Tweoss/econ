use actix::prelude::*;

#[derive(Message)]
#[rtype(result="()")]
pub struct CloseGame {
    pub game_id: String,
}