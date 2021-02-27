use actix::prelude::*;

#[derive(Message)]
#[rtype(result="()")]
pub struct EndedGame {
}

#[derive(Message)]
#[rtype(result="()")]
pub struct NextTurn {}