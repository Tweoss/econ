use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct EndGame {
    pub game_id: String,
}
