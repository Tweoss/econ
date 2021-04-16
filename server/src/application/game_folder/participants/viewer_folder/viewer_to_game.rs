use crate::application::game_folder::participants::viewer_folder::viewer::Viewer;
use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterAddressGetInfo {
	pub name: String,
	pub addr: Addr<Viewer>,
}
