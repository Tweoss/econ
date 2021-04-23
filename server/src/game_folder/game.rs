use crate::participants::{
	consumer_folder::{consumer::ConsumerState, consumer_structs, consumer_to_game},
	director_folder::{director::DirectorState, director_structs, director_to_game},
	producer_folder::{producer::ProducerState, producer_structs, producer_to_game},
	viewer_folder::{viewer::ViewerState, viewer_structs, viewer_to_game},
};
use actix::prelude::*;

use crate::application::app::AppState;
use crate::application::app_to_game::*;
use crate::game_folder::{
	game_to_app, game_to_consumer, game_to_director, game_to_participant, game_to_producer,
	game_to_viewer,
};
use crate::participants::participant_to_game;

use rand::random;
use sha256::digest;
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::RwLock;

const INITIAL_BALANCE: f64 = 4000.;

pub struct Game {
	consumers: RwLock<HashMap<String, ConsumerState>>,
	producers: RwLock<HashMap<String, ProducerState>>,
	directors: RwLock<HashMap<String, DirectorState>>,
	viewers: RwLock<HashMap<String, ViewerState>>,
	past_turn: RwLock<HashMap<String, producer_structs::Participant>>,
	name_main_director: String,
	state_main_director: DirectorState,
	is_open: bool,
	turn: u64,
	trending: u8,
	supply_shock: u8,
	subsidies: u8,
	app_addr: Addr<AppState>,
	producer_next: bool,
	game_id: String,
}

impl Actor for Game {
	type Context = Context<Self>;
	fn stopping(&mut self, _: &mut Self::Context) -> Running {
		println!(
			"Stopping a game actor with main director being: {} and gameid: {}",
			self.name_main_director, self.game_id
		);
		let date = chrono::Local::now();
		println!("Date and time: {}", date.format("[%Y-%m-%d][%H:%M:%S]"));
		Running::Stop
	}
}

impl Game {
	pub fn new(
		app_addr: Addr<AppState>,
		id_main_director: String,
		name_main_director: String,
		game_id: String,
	) -> Game {
		println!(
			"Making a new GAME with director name: {}",
			name_main_director
		);
		Game {
			producers: RwLock::new(HashMap::new()),
			consumers: RwLock::new(HashMap::new()),
			directors: RwLock::new(HashMap::new()),
			viewers: RwLock::new(HashMap::new()),
			past_turn: RwLock::new(HashMap::new()),
			name_main_director,
			state_main_director: DirectorState::new(id_main_director),
			is_open: false,
			trending: 0,
			supply_shock: 0,
			subsidies: 0,
			turn: 1,
			app_addr,
			producer_next: true,
			game_id,
		}
	}
	fn get_director_info(&self) -> director_structs::Info {
		let mut consumers = Vec::new();
		let mut producers = Vec::new();
		let mut directors = Vec::new();
		let mut viewers = Vec::new();

		for consumer in self.consumers.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if consumer.1.addr != None {
				if consumer.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				} else {
					state = director_structs::PlayerState::Unresponsive;
				}
			} else {
				state = director_structs::PlayerState::Disconnected;
			}
			consumers.push((
				consumer.0.clone(),
				director_structs::Participant {
					state,
					took_turn: Some(consumer.1.took_turn),
					id: consumer.1.id.clone(),
				},
			))
		}

		for producer in self.producers.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if producer.1.addr != None {
				if producer.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				} else {
					state = director_structs::PlayerState::Unresponsive;
				}
			} else {
				state = director_structs::PlayerState::Disconnected;
			}
			producers.push((
				producer.0.clone(),
				director_structs::Participant {
					state,
					took_turn: Some(producer.1.took_turn),
					id: producer.1.id.clone(),
				},
			))
		}

		for viewer in self.viewers.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if viewer.1.addr != None {
				if viewer.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				} else {
					state = director_structs::PlayerState::Unresponsive;
				}
			} else {
				state = director_structs::PlayerState::Disconnected;
			}
			viewers.push((
				viewer.0.clone(),
				director_structs::Participant {
					state,
					took_turn: None,
					id: viewer.1.id.clone(),
				},
			))
		}

		for director in self.directors.read().unwrap().iter() {
			let state: director_structs::PlayerState;
			if director.1.addr != None {
				if director.1.is_responsive {
					state = director_structs::PlayerState::Connected;
				} else {
					state = director_structs::PlayerState::Unresponsive;
				}
			} else {
				state = director_structs::PlayerState::Disconnected;
			}
			directors.push((
				director.0.clone(),
				director_structs::Participant {
					state,
					took_turn: None,
					id: director.1.id.clone(),
				},
			))
		}
		director_structs::Info {
			consumers,
			producers,
			directors,
			viewers,
			is_open: self.is_open,
			turn: self.turn,
			trending: self.trending,
			supply_shock: self.supply_shock,
			subsidies: self.subsidies,
			game_id: self.game_id.clone(),
		}
	}
	fn get_producer_info(&self, name: String) -> producer_structs::Info {
		let producers = self.past_turn.read().unwrap().clone().into_iter().collect();
		let producers_list = self.producers.read().unwrap();
		let producer = producers_list.get(&name).unwrap();
		producer_structs::Info {
			producers,
			turn: self.turn,
			game_id: self.game_id.clone(),
			supply_shock: self.supply_shock,
			subsidies: self.subsidies,
			balance: producer.balance,
			score: producer.score,
			took_turn: producer.took_turn,
		}
	}
	fn get_consumer_info(&self, name: String) -> consumer_structs::Info {
		let producers = if self.turn % 2 == 0 {
			self.past_turn.read().unwrap().clone().into_iter().collect()
		} else {
			Vec::new()
		};
		let consumers_list = self.consumers.read().unwrap();
		let consumer = consumers_list.get(&name).unwrap();
		consumer_structs::Info {
			producers,
			turn: self.turn,
			game_id: self.game_id.clone(),
			trending: self.trending,
			balance: consumer.balance,
			quantity_purchased: consumer.quantity_purchased,
			score: consumer.score,
			took_turn: consumer.took_turn,
		}
	}
	fn get_viewer_info(&self) -> viewer_structs::Info {
		let participants: Vec<viewer_structs::Participant> = self
			.consumers
			.read()
			.unwrap()
			.iter()
			.map(|(name, state)| viewer_structs::Participant {
				name: name.clone(),
				is_consumer: true,
				score: state.score,
				next_index: 0,
			})
			.chain(self.producers.read().unwrap().iter().map(|(name, state)| {
				viewer_structs::Participant {
					name: name.clone(),
					is_consumer: false,
					score: state.score,
					next_index: 0,
				}
			}))
			.collect::<Vec<viewer_structs::Participant>>();

		viewer_structs::Info {
			participants,
			turn: self.turn,
			game_id: self.game_id.clone(),
			is_open: self.is_open,
			trending: self.trending,
			subsidies: self.subsidies,
			supply_shock: self.supply_shock,
		}
	}
	// * returns purchased, expense, balance, how much was purchased
	// ! balance may be unnecessary for consumer actor to know
	fn purchase(
		&self,
		user_id: &str,
		targets: Vec<(String, f64)>,
	) -> (f64, f64, f64, Vec<(String, f64)>) {
		let mut producers = self.past_turn.write().unwrap();
		let mut consumers = self.consumers.write().unwrap();
		let consumer = consumers.get_mut(user_id).unwrap();
		let mut purchased = 0.;
		let mut expense = 0.;
		let mut return_targets = Vec::new();
		for target in targets {
			// * if they try to purchase negative quantity stop
			if target.1 <= 0. {
				break;
			}
			if let Some(producer) = producers.get_mut(&target.0) {
				// * if the consumer has enough money and there is enough quantity
				if consumer.balance >= target.1 * producer.price && producer.remaining >= target.1 {
					let single_quantity = target.1;
					purchased += single_quantity;
					producer.remaining -= single_quantity;
					expense += single_quantity * producer.price;
					consumer.balance -= expense;
					return_targets.push((target.0, single_quantity));
				}
				// * if quantity requested > remaning and there is enough money, purchase all of it
				else if target.1 > producer.remaining
					&& consumer.balance >= producer.remaining * producer.price
				{
					let single_quantity = producer.remaining;
					purchased += single_quantity;
					producer.remaining = 0.;
					expense += single_quantity * producer.price;
					consumer.balance -= expense;
					return_targets.push((target.0, single_quantity));
				}
				// * if there is not enough money but enough quantity, purchase as much as possible given balance
				else if consumer.balance < target.1 * producer.price
					&& target.1 < producer.remaining
				{
					let single_quantity = consumer.balance / producer.price;
					purchased += single_quantity;
					producer.remaining -= single_quantity;
					expense += consumer.balance;
					consumer.balance = 0.;
					return_targets.push((target.0, single_quantity));
					break;
				}
				// * if there is not enough money AND not enough quantity, probably trying to exploit. don't do anything
			}
		}
		println!(
			"Game says Purchased: {}, expense: {}, resulting_balance: {}",
			purchased, expense, consumer.balance
		);
		(purchased, expense, consumer.balance, return_targets)
	}
	fn get_top_scores(&self) -> [Option<Vec<(String, f64, bool)>>; 3] {
		let reading = (
			self.consumers.read().unwrap(),
			self.producers.read().unwrap(),
		);
		let mut name_score_consumer: Vec<(String, f64, bool)> = Vec::new();
		for p in reading.0.iter() {
			name_score_consumer.push((p.0.clone(), p.1.score, true));
		}
		for p in reading.1.iter() {
			name_score_consumer.push((p.0.clone(), p.1.score, false));
		}
		if name_score_consumer.is_empty() {
			return [None, None, None];
		}
		name_score_consumer.sort_by(|(_, a, _), (_, b, _)| b.partial_cmp(a).unwrap());
		let mut output: [Option<Vec<(String, f64, bool)>>; 3] = [None, None, None];
		let mut current_index = 0;
		let mut current_max = name_score_consumer[0].1;
		for p in name_score_consumer {
			if (p.1 - current_max).abs() < f32::EPSILON.into() {
				if let Some(vec) = &mut output[current_index] {
					vec.push((p.0, p.1, p.2));
				} else {
					output[current_index] = Some(vec![(p.0, p.1, p.2)]);
				}
			} else if p.1 - current_max < 0. {
				if current_index < 2 {
					current_max = p.1;
					current_index += 1;
					output[current_index] = Some(vec![(p.0, p.1, p.2)]);
				} else {
					break;
				}
			}
		}
		output
	}
	fn generate_and_send_win(&self) {
		let list = self.get_top_scores();
		println!("{:?}", list);
		let mut hash_list: [Option<Vec<(String, String)>>; 3] = Default::default();
		for (index, array_element) in list.iter().enumerate() {
			if let Some(vec) = &array_element {
				for p in vec {
					let hash = digest(format!("{} {} {} {}", random::<u8>(), p.0, p.1, p.2))
						.split_at(10)
						.0
						.to_string();
					if let Some(hash_vec) = &mut hash_list[index] {
						hash_vec.push((p.0.clone(), hash.clone()));
					} else {
						hash_list[index] = Some(vec![(p.0.clone(), hash.clone())])
					}
					if p.2 {
						if let Some(addr) = &self.consumers.read().unwrap().get(&p.0).unwrap().addr
						{
							addr.do_send(game_to_participant::Winner {
								hash,
								place: (index + 1).try_into().unwrap(),
							});
						}
					} else if let Some(addr) =
						&self.producers.read().unwrap().get(&p.0).unwrap().addr
					{
						addr.do_send(game_to_participant::Winner {
							hash,
							place: (index + 1).try_into().unwrap(),
						});
					}
				}
			}
		}
		let view_list = list.iter().map(|each| {
			if let Some(vec) = each {
				let score = vec[0].1;
				Some((
					vec.iter()
						.map(|(name, _, _)| (name.clone()))
						.collect::<Vec<String>>(),
						score
				))
			} else {
				None
			}
		}).collect::<Vec<Option<(Vec<String>, f64)>>>();
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Winners {
				array: hash_list.clone(),
			})
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Winners {
					array: hash_list.clone(),
				})
			}
		}
		for elem in self.viewers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_viewer::Winners {
					vector: view_list.clone(),
				})
			}
		}
	}
}

// ! APPLICATION TO GAME HANDLERS

/// Handler for NewPlayer message.
///
/// Register a NewPlayer
impl Handler<NewPlayer> for Game {
	type Result = String;
	fn handle(&mut self, msg: NewPlayer, _: &mut Context<Self>) -> Self::Result {
		if self
			.consumers
			.read()
			.unwrap()
			.keys()
			.any(|x| x == &msg.username)
			|| self
				.producers
				.read()
				.unwrap()
				.keys()
				.any(|x| x == &msg.username)
		{
			return "Name taken".to_string();
		}
		self.producer_next = !self.producer_next;
		if self.producer_next {
			self.consumers.write().unwrap().insert(
				msg.username.clone(),
				ConsumerState::new(msg.user_id.clone()),
			);
			for elem in self.directors.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_director::NewParticipant {
						id: msg.user_id.clone(),
						name: msg.username.clone(),
						participant_type: director_structs::ParticipantType::Consumer,
					});
				}
			}
			for elem in self.viewers.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_viewer::NewParticipant {
						name: msg.username.clone(),
						is_consumer: true,
					});
				}
			}
			if let Some(addr) = &self.state_main_director.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id,
					name: msg.username,
					participant_type: director_structs::ParticipantType::Consumer,
				})
			}
			"consumer".to_string()
		} else {
			self.producers.write().unwrap().insert(
				msg.username.clone(),
				ProducerState::new(msg.user_id.clone()),
			);
			for elem in self.directors.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_director::NewParticipant {
						id: msg.user_id.clone(),
						name: msg.username.clone(),
						participant_type: director_structs::ParticipantType::Producer,
					});
				}
			}
			for elem in self.viewers.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_viewer::NewParticipant {
						name: msg.username.clone(),
						is_consumer: false,
					});
				}
			}
			if let Some(addr) = &self.state_main_director.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id,
					name: msg.username,
					participant_type: director_structs::ParticipantType::Producer,
				})
			}
			"producer".to_string()
		}
	}
}

/// Return if the Game is open to players
impl Handler<IsGameOpen> for Game {
	type Result = bool;
	fn handle(&mut self, _: IsGameOpen, _: &mut Context<Self>) -> Self::Result {
		self.is_open
	}
}

/// Register an additional director
impl Handler<NewDirector> for Game {
	type Result = ();
	fn handle(&mut self, msg: NewDirector, _: &mut Context<Self>) -> Self::Result {
		if self
			.directors
			.read()
			.unwrap()
			.keys()
			.any(|x| x == &msg.username)
		{
			return;
		}
		self.directors.write().unwrap().insert(
			msg.username.clone(),
			DirectorState::new(msg.user_id.clone()),
		);
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id.clone(),
					name: msg.username.clone(),
					participant_type: director_structs::ParticipantType::Director,
				});
			}
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::NewParticipant {
				id: msg.user_id,
				name: msg.username,
				participant_type: director_structs::ParticipantType::Director,
			});
		}
	}
}

/// Register an additional viewer
impl Handler<NewViewer> for Game {
	type Result = bool;
	fn handle(&mut self, msg: NewViewer, _: &mut Context<Self>) -> Self::Result {
		if self
			.viewers
			.read()
			.unwrap()
			.keys()
			.any(|x| x == &msg.username)
		{
			false
		} else {
			self.viewers
				.write()
				.unwrap()
				.insert(msg.username.clone(), ViewerState::new(msg.user_id.clone()));
			for elem in self.directors.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_director::NewParticipant {
						id: msg.user_id.clone(),
						name: msg.username.clone(),
						participant_type: director_structs::ParticipantType::Viewer,
					});
				}
			}
			if let Some(addr) = &self.state_main_director.addr {
				addr.do_send(game_to_director::NewParticipant {
					id: msg.user_id,
					name: msg.username,
					participant_type: director_structs::ParticipantType::Viewer,
				});
			}
			true
		}
	}
}

/// Check if this director is registered
impl Handler<IsDirector> for Game {
	type Result = Option<String>;
	fn handle(&mut self, msg: IsDirector, _: &mut Context<Self>) -> Self::Result {
		println!("Asked if IsDirector for a game.");
		if let Some((name, _)) = self
			.directors
			.read()
			.unwrap()
			.iter()
			.find(|(_, s)| s.id == msg.user_id)
		{
			Some(name.to_string())
		} else if self.state_main_director.id == msg.user_id {
			Some(self.name_main_director.clone())
		} else {
			None
		}
	}
}

/// Check if this consumer or producer is registered
impl Handler<IsPlayer> for Game {
	type Result = Option<String>;
	fn handle(&mut self, msg: IsPlayer, _: &mut Context<Self>) -> Self::Result {
		if let Some((name, _)) = self
			.consumers
			.read()
			.unwrap()
			.iter()
			.find(|(_, s)| s.id == msg.user_id)
		{
			Some(name.to_string())
		} else if let Some((name, _)) = self
			.producers
			.read()
			.unwrap()
			.iter()
			.find(|(_, s)| s.id == msg.user_id)
		{
			Some(name.to_string())
		} else {
			None
		}
	}
}

impl Handler<IsViewer> for Game {
	type Result = Option<String>;
	fn handle(&mut self, msg: IsViewer, _: &mut Context<Self>) -> Self::Result {
		if let Some((name, _)) = self
			.viewers
			.read()
			.unwrap()
			.iter()
			.find(|(_, s)| s.id == msg.user_id)
		{
			Some(name.to_string())
		} else {
			None
		}
	}
}

impl Handler<IsMainDirector> for Game {
	type Result = bool;
	fn handle(&mut self, msg: IsMainDirector, _: &mut Context<Self>) -> Self::Result {
		self.state_main_director.id == msg.user_id
	}
}

// ! WEBSOCKET TO GAME HANDLERS

impl Handler<director_to_game::EndGame> for Game {
	type Result = ();
	fn handle(&mut self, _msg: director_to_game::EndGame, ctx: &mut Context<Self>) -> Self::Result {
		self.generate_and_send_win();
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_participant::EndedGame {});
		}
		for director in self.directors.read().unwrap().values() {
			if let Some(addr) = &director.addr {
				addr.do_send(game_to_participant::EndedGame {});
			}
		}
		for consumer in self.consumers.read().unwrap().values() {
			if let Some(addr) = &consumer.addr {
				addr.do_send(game_to_participant::EndedGame {});
			}
		}
		for producer in self.producers.read().unwrap().values() {
			if let Some(addr) = &producer.addr {
				addr.do_send(game_to_participant::EndedGame {});
			}
		}
		for viewer in self.viewers.read().unwrap().values() {
			if let Some(addr) = &viewer.addr {
				addr.do_send(game_to_participant::EndedGame {});
			}
		}
		self.app_addr.do_send(game_to_app::EndGame {
			game_id: self.game_id.clone(),
		});
		ctx.stop();
	}
}

impl Handler<director_to_game::RegisterAddressGetInfo> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: director_to_game::RegisterAddressGetInfo,
		_: &mut Context<Self>,
	) -> Self::Result {
		if msg.name == self.name_main_director {
			self.state_main_director.addr = Some(msg.addr.clone());
		} else if let Some(mut addr_value) = self.directors.write().unwrap().get_mut(&msg.name) {
			addr_value.addr = Some(msg.addr.clone());
		}
		msg.addr.do_send(game_to_director::Info {
			info: self.get_director_info(),
		});
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Connected {
				name: msg.name.clone(),
				participant_type: "director".to_string(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Connected {
					name: msg.name.clone(),
					participant_type: "director".to_string(),
				});
			}
		}
	}
}

impl Handler<director_to_game::KickParticipant> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: director_to_game::KickParticipant,
		_: &mut Context<Self>,
	) -> Self::Result {
		self.consumers
			.write()
			.unwrap()
			.remove_entry(&msg.name)
			.map(|x| {
				x.1.addr
					.map(|addr| addr.do_send(game_to_participant::Kicked {}))
			});
		self.producers
			.write()
			.unwrap()
			.remove_entry(&msg.name)
			.map(|x| {
				x.1.addr
					.map(|addr| addr.do_send(game_to_participant::Kicked {}))
			});
		self.directors
			.write()
			.unwrap()
			.remove_entry(&msg.name)
			.map(|x| {
				x.1.addr
					.map(|addr| addr.do_send(game_to_participant::Kicked {}))
			});
		self.viewers
			.write()
			.unwrap()
			.remove_entry(&msg.name)
			.map(|x| {
				x.1.addr
					.map(|addr| addr.do_send(game_to_participant::Kicked {}))
			});
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::KickedParticipant {
					name: msg.name.clone(),
				});
			};
		}
		for elem in self.viewers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_viewer::KickedParticipant {
					name: msg.name.clone(),
				});
			};
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::KickedParticipant { name: msg.name });
		}
	}
}

impl Handler<director_to_game::OpenGame> for Game {
	type Result = ();
	fn handle(&mut self, _msg: director_to_game::OpenGame, _: &mut Context<Self>) {
		self.is_open = true;
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::GameOpened {});
			}
		}
		for viewer in self.viewers.read().unwrap().values() {
			if let Some(addr) = &viewer.addr {
				addr.do_send(game_to_viewer::GameOpened {});
			}
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::GameOpened {});
		}
	}
}

impl Handler<director_to_game::CloseGame> for Game {
	type Result = ();
	fn handle(&mut self, _msg: director_to_game::CloseGame, _: &mut Context<Self>) {
		self.is_open = false;
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::GameClosed {});
			}
		}
		for viewer in self.viewers.read().unwrap().values() {
			if let Some(addr) = &viewer.addr {
				addr.do_send(game_to_viewer::GameClosed {});
			}
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::GameClosed {});
		}
	}
}

impl Handler<director_to_game::SetOffsets> for Game {
	type Result = ();
	fn handle(&mut self, msg: director_to_game::SetOffsets, _: &mut Context<Self>) {
		// * Turn starts at 1, producer.
		if self.turn % 2 == 1
			&& self.subsidies == msg.subsidies
			&& self.supply_shock == msg.supply_shock
			|| self.turn % 2 == 0 && self.trending == msg.trending
		{
			// * make sure that the others aren't changing
			self.subsidies = msg.subsidies;
			self.trending = msg.trending;
			self.supply_shock = msg.supply_shock;
			if let Some(addr) = &self.state_main_director.addr {
				addr.do_send(game_to_participant::NewOffsets {
					subsidies: msg.subsidies,
					trending: msg.trending,
					supply_shock: msg.supply_shock,
				});
			}
			for elem in self.directors.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_participant::NewOffsets {
						subsidies: msg.subsidies,
						trending: msg.trending,
						supply_shock: msg.supply_shock,
					});
				}
			}
			for elem in self.producers.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_participant::NewOffsets {
						subsidies: msg.subsidies,
						trending: msg.trending,
						supply_shock: msg.supply_shock,
					});
				}
			}
			for elem in self.consumers.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_participant::NewOffsets {
						subsidies: msg.subsidies,
						trending: msg.trending,
						supply_shock: msg.supply_shock,
					});
				}
			}
			for elem in self.viewers.read().unwrap().values() {
				if let Some(addr) = &elem.addr {
					addr.do_send(game_to_participant::NewOffsets {
						subsidies: msg.subsidies,
						trending: msg.trending,
						supply_shock: msg.supply_shock,
					});
				}
			}
		}
	}
}

impl Handler<director_to_game::ForceTurn> for Game {
	type Result = ();
	fn handle(&mut self, _: director_to_game::ForceTurn, _: &mut Context<Self>) -> Self::Result {
		self.turn += 1;
		let mut new_scores: Vec<(String, f64)> = Vec::new();
		if self.turn % 2 == 0 {
			let list = self.past_turn.read().unwrap().clone();
			for producer in self.producers.write().unwrap().iter_mut() {
				if let Some(addr) = &producer.1.addr {
					addr.do_send(game_to_producer::TurnList {
						list: list.clone().into_iter().collect(),
					});
				}
				producer.1.took_turn = false;
				producer.1.score += producer.1.balance;
				if producer.1.balance != 0. {
					new_scores.push((producer.0.clone(), producer.1.score));
				}
				producer.1.balance = 0.;
			}
			for consumer in self.consumers.write().unwrap().values_mut() {
				consumer.took_turn = false;
				consumer.balance = INITIAL_BALANCE;
				if let Some(addr) = &consumer.addr {
					addr.do_send(game_to_consumer::TurnList {
						list: list.clone().into_iter().collect(),
					});
				}
			}
		} else {
			self.past_turn.write().unwrap().clear();
			for consumer in self.consumers.write().unwrap().iter_mut() {
				consumer.1.score += consumer.1.balance;
				if consumer.1.balance != 0. {
					new_scores.push((consumer.0.clone(), consumer.1.score));
				}
				consumer.1.balance = 0.;
			}
			for producer in self.producers.write().unwrap().values_mut() {
				producer.took_turn = false;
				producer.balance = INITIAL_BALANCE
			}
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_participant::TurnAdvanced {});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_participant::TurnAdvanced {});
			}
		}
		for elem in self.producers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_participant::TurnAdvanced {});
			}
		}
		for elem in self.consumers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_participant::TurnAdvanced {});
			}
		}
		for elem in self.viewers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_participant::TurnAdvanced {});
				addr.do_send(game_to_viewer::NewScores {
					list: new_scores.clone(),
				});
			}
		}
	}
}

impl Handler<producer_to_game::RegisterAddressGetInfo> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: producer_to_game::RegisterAddressGetInfo,
		_: &mut Context<Self>,
	) -> Self::Result {
		if let Some(mut addr_value) = self.producers.write().unwrap().get_mut(&msg.name) {
			addr_value.addr = Some(msg.addr.clone());
		}
		msg.addr.do_send(game_to_producer::Info {
			info: self.get_producer_info(msg.name.clone()),
		});
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Connected {
				name: msg.name.clone(),
				participant_type: "producer".to_string(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Connected {
					name: msg.name.clone(),
					participant_type: "producer".to_string(),
				});
			}
		}
	}
}

impl Handler<producer_to_game::NewScoreEndTurn> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: producer_to_game::NewScoreEndTurn,
		_: &mut Context<Self>,
	) -> Self::Result {
		self.producers
			.write()
			.unwrap()
			.get_mut(&msg.name)
			.unwrap()
			.score = msg.new_score;
		self.producers
			.write()
			.unwrap()
			.get_mut(&msg.name)
			.unwrap()
			.balance = 0.;
		self.producers
			.write()
			.unwrap()
			.get_mut(&msg.name)
			.unwrap()
			.took_turn = true;
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::TurnTaken {
				name: msg.name.clone(),
				participant_type: "producer".to_string(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::TurnTaken {
					name: msg.name.clone(),
					participant_type: "producer".to_string(),
				});
			}
		}
		for elem in self.viewers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_viewer::NewScores {
					list: vec![(msg.name.clone(), msg.new_score)],
				});
			}
		}
		self.past_turn.write().unwrap().insert(
			msg.name,
			producer_structs::Participant {
				produced: msg.produced,
				remaining: msg.produced,
				price: msg.price,
			},
		);
	}
}

impl Handler<consumer_to_game::RegisterAddressGetInfo> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: consumer_to_game::RegisterAddressGetInfo,
		_: &mut Context<Self>,
	) -> Self::Result {
		if let Some(mut addr_value) = self.consumers.write().unwrap().get_mut(&msg.name) {
			addr_value.addr = Some(msg.addr.clone());
		}
		msg.addr.do_send(game_to_consumer::Info {
			info: self.get_consumer_info(msg.name.clone()),
		});
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Connected {
				name: msg.name.clone(),
				participant_type: "consumer".to_string(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Connected {
					name: msg.name.clone(),
					participant_type: "consumer".to_string(),
				});
			}
		}
	}
}

impl Handler<consumer_to_game::NewScoreEndTurn> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: consumer_to_game::NewScoreEndTurn,
		_: &mut Context<Self>,
	) -> Self::Result {
		let mut consumers = self.consumers.write().unwrap();
		let consumer = consumers.get_mut(&msg.name).unwrap();
		consumer.score = msg.new_score;
		consumer.balance = 0.;
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::TurnTaken {
				name: msg.name.clone(),
				participant_type: "consumer".to_string(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::TurnTaken {
					name: msg.name.clone(),
					participant_type: "consumer".to_string(),
				});
			}
		}
		for elem in self.viewers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_viewer::NewScores {
					list: vec![(msg.name.clone(), msg.new_score)],
				});
			}
		}
	}
}

impl Handler<consumer_to_game::TryChoice> for Game {
	type Result = ();
	fn handle(&mut self, msg: consumer_to_game::TryChoice, _: &mut Context<Self>) {
		let (purchased, expense, balance, targets) = self.purchase(&msg.name, msg.elements);
		self.consumers
			.read()
			.unwrap()
			.get(&msg.name)
			.unwrap()
			.addr
			.as_ref()
			.unwrap()
			.do_send(game_to_consumer::PurchaseResult {
				expense,
				balance,
				purchased,
			});
		for target in targets.iter() {
			if let Some(producer) = self.producers.write().unwrap().get_mut(&target.0) {
				let addition =
					target.1 * self.past_turn.read().unwrap().get(&target.0).unwrap().price;
				println!(
					"Target.1 = {}, price = {}, addition = {}",
					target.1,
					self.past_turn.read().unwrap().get(&target.0).unwrap().price,
					addition
				);
				producer.score += addition;
				if let Some(addr) = &producer.addr {
					addr.do_send(game_to_producer::GotPurchased {
						additional_score: target.1 * addition,
					});
				}
				for elem in self.viewers.read().unwrap().values() {
					if let Some(addr) = &elem.addr {
						addr.do_send(game_to_viewer::NewScores {
							list: vec![(target.0.clone(), producer.score)],
						});
					}
				}
			}
		}
		for elem in self.producers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_participant::StockReduced {
					targets: targets.clone(),
				});
			}
		}
		for elem in self.consumers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_participant::StockReduced {
					targets: targets.clone(),
				});
			}
		}
	}
}

impl Handler<consumer_to_game::NewScoreCalculated> for Game {
	type Result = ();
	fn handle(&mut self, msg: consumer_to_game::NewScoreCalculated, _: &mut Context<Self>) {
		if let Some(consumer) = self.consumers.write().unwrap().get_mut(&msg.name) {
			consumer.score = msg.new_score;
		}
		for elem in self.viewers.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_viewer::NewScores {
					list: vec![(msg.name.clone(), msg.new_score)],
				});
			}
		}
	}
}

impl Handler<viewer_to_game::RegisterAddressGetInfo> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: viewer_to_game::RegisterAddressGetInfo,
		_: &mut Context<Self>,
	) -> Self::Result {
		if let Some(mut addr_value) = self.viewers.write().unwrap().get_mut(&msg.name) {
			addr_value.addr = Some(msg.addr.clone());
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Connected {
				name: msg.name.clone(),
				participant_type: "viewer".to_string(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Connected {
					name: msg.name.clone(),
					participant_type: "viewer".to_string(),
				});
			}
		}
		msg.addr.do_send(game_to_viewer::Info {
			info: self.get_viewer_info(),
		});
	}
}

impl Handler<participant_to_game::Unresponsive> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: participant_to_game::Unresponsive,
		_: &mut Context<Self>,
	) -> Self::Result {
		println!("Unresponsive {}: {}", &msg.participant_type, &msg.name);
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Unresponsive {
				name: msg.name.clone(),
				participant_type: msg.participant_type.clone(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Unresponsive {
					name: msg.name.clone(),
					participant_type: msg.participant_type.clone(),
				});
			}
		}
	}
}

impl Handler<participant_to_game::Responsive> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: participant_to_game::Responsive,
		_: &mut Context<Self>,
	) -> Self::Result {
		println!("Responsive again");
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Connected {
				name: msg.name.clone(),
				participant_type: msg.participant_type.clone(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Connected {
					name: msg.name.clone(),
					participant_type: msg.participant_type.clone(),
				});
			}
		}
	}
}

impl Handler<participant_to_game::Disconnected> for Game {
	type Result = ();
	fn handle(
		&mut self,
		msg: participant_to_game::Disconnected,
		_: &mut Context<Self>,
	) -> Self::Result {
		match msg.participant_type.as_str() {
			"director" => {
				if msg.name == self.name_main_director {
					self.state_main_director.addr = None;
				} else {
					self.directors
						.write()
						.unwrap()
						.get_mut(&msg.name)
						.unwrap()
						.addr = None;
				}
			}
			"consumer" => {
				self.consumers
					.write()
					.unwrap()
					.get_mut(&msg.name)
					.unwrap()
					.addr = None;
			}
			"producer" => {
				self.producers
					.write()
					.unwrap()
					.get_mut(&msg.name)
					.unwrap()
					.addr = None;
			}
			"viewer" => {
				self.viewers
					.write()
					.unwrap()
					.get_mut(&msg.name)
					.unwrap()
					.addr = None;
			}
			_ => (),
		}
		if let Some(addr) = &self.state_main_director.addr {
			addr.do_send(game_to_director::Disconnected {
				name: msg.name.clone(),
				participant_type: msg.participant_type.clone(),
			});
		}
		for elem in self.directors.read().unwrap().values() {
			if let Some(addr) = &elem.addr {
				addr.do_send(game_to_director::Disconnected {
					name: msg.name.clone(),
					participant_type: msg.participant_type.clone(),
				});
			}
		}
	}
}
