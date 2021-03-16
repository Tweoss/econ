use actix::prelude::*;
// use std::sync::Mutex;
use actix::StreamHandler;
use actix_web_actors::ws;
use std::time::Instant;

// use crate::application::other_messages;

use crate::application::game_folder::game::Game;
use crate::application::game_folder::game_to_participant;
use crate::application::game_folder::game_to_producer;

use crate::application::game_folder::participants::participant_to_game;
use crate::application::game_folder::participants::producer_folder::producer_to_game;
use serde_cbor::{from_slice, to_vec};

use crate::application::game_folder::participants::producer_folder::producer_structs::{
	self, ProducerClientMsg, ProducerClientType, ProducerServerMsg, ProducerServerType,
	ServerExtraFields,
};

use crate::application::game_folder::participants::json::{
	CLIENT_TERMINATE, CLIENT_TIMEOUT, HEARTBEAT_INTERVAL,
};

pub struct ProducerState {
	pub is_responsive: bool,
	pub took_turn: bool,
	pub score: f64,
	pub balance: f64,
	pub quantity_remaining: f64,
	pub price: f64,
	pub addr: Option<Addr<Producer>>,
}

impl ProducerState {
	pub fn new() -> ProducerState {
		ProducerState {
			is_responsive: true,
			took_turn: false,
			score: 0.,
			balance: 0.,
			quantity_remaining: 0.,
			price: 0.,
			addr: None,
		}
	}
}

pub struct Producer {
	pub uuid: String,
	pub game_id: String,
	pub game_addr: Addr<Game>,
	hb: Instant,
	is_unresponsive: bool,
}

impl Actor for Producer {
	type Context = ws::WebsocketContext<Self>;
	fn started(&mut self, ctx: &mut Self::Context) {
		self.game_addr
			.do_send(producer_to_game::RegisterAddressGetInfo {
				user_id: self.uuid.clone(),
				addr: ctx.address(),
			});
		self.hb(ctx);
	}
	fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
		println!(
			"Stopping a producer actor: {} and {}",
			self.game_id, self.uuid
		);
		self.game_addr.do_send(participant_to_game::Disconnected {
			id: self.uuid.clone(),
			participant_type: "producer".to_owned(),
		});
		ctx.terminate();
		Running::Stop
	}
}

impl Producer {
	// pub async fn new(uuid: String, game_id: String, addr: &actix_web::web::Data<Addr<AppState>>) -> Option<Producer> {
	// 	if let Some(game_addr) = addr.send(IsProducer {user_id: uuid.clone(), game_id: game_id.clone()}).await.unwrap() {
	// 		Some(Producer {uuid, game_id, game_addr})
	// 	}
	// 	else {
	// 		None
	// 	}
	// }
	fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
		ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
			// check client heartbeats
			if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
				// heartbeat timed out
				// notify game
				act.game_addr.do_send(participant_to_game::Unresponsive {
					id: act.uuid.clone(),
					participant_type: "producer".to_owned(),
				});
				if Instant::now().duration_since(act.hb) > CLIENT_TERMINATE {
					ctx.binary(
						to_vec(&ProducerServerMsg {
							msg_type: ProducerServerType::ServerKicked,
							extra_fields: None,
						})
						.unwrap(),
					);
					act.game_addr.do_send(participant_to_game::Disconnected {
						id: act.uuid.clone(),
						participant_type: "producer".to_owned(),
					});
					ctx.stop();
				}
				act.is_unresponsive = true;
			}
			let ping = to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::Ping,
				extra_fields: None,
			})
			.unwrap();
			ctx.binary(ping);
		});
	}
	fn reset_hb(&mut self) {
		self.hb = Instant::now();
		if self.is_unresponsive {
			self.game_addr.do_send(participant_to_game::Responsive {
				id: self.uuid.clone(),
				participant_type: "producer".to_string(),
			});
			self.is_unresponsive = false;
		}
	}
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Producer {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		if let Ok(ws::Message::Binary(bin)) = msg {
  				if let Ok(message) =
  					from_slice::<ProducerClientMsg>(&bin.to_vec()) {
  					println!("{:?}", message);
  					match message.msg_type {
  						ProducerClientType::Choice => (),
  						ProducerClientType::Pong => (),
  						// _ => (),
  					}

  				}
  				else {
  					println!("Invalid structure received");
  				}
  			}
		self.reset_hb();
	}
}

impl Handler<game_to_participant::EndedGame> for Producer {
	type Result = ();
	fn handle(
		&mut self,
		_msg: game_to_participant::EndedGame,
		ctx: &mut Self::Context,
	) -> Self::Result {
		ctx.stop();
	}
}

impl Handler<game_to_producer::Info> for Producer {
	type Result = ();
	fn handle(&mut self, msg: game_to_producer::Info, ctx: &mut Self::Context) -> Self::Result {
		let extra_fields = producer_structs::ServerExtraFields {
			info: Some(msg.info),
			..Default::default()
		};
		ctx.binary(
			to_vec(&ProducerServerMsg {
				msg_type: ProducerServerType::Info,
				extra_fields: Some(extra_fields),
			})
			.unwrap(),
		);
	}
}
