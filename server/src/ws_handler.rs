use actix_web::HttpMessage;
use actix_web::Responder;
use actix_web::{http, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::application::app::AppState;
use crate::participants::consumer_folder::consumer::Consumer;
use crate::participants::director_folder::director::Director;
use crate::participants::producer_folder::producer::Producer;
use crate::participants::viewer_folder::viewer::Viewer;

use crate::handle_to_app;

pub async fn handle_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
	let viewtype: String = req.match_info().get("viewtype").unwrap().parse().unwrap();
	let game_id: String = req.match_info().get("game_id").unwrap().parse().unwrap();
	let uuid: String = req.match_info().get("uuid").unwrap().parse().unwrap();
	let addr = req.app_data::<web::Data<actix::Addr<AppState>>>().unwrap();

	match viewtype.as_ref() {
		"director" => {
			if let Some((game_addr, name)) = addr
				.send(handle_to_app::IsRegisteredDirector {
					user_id: uuid.clone(),
					game_id: game_id.clone(),
				})
				.await
				.unwrap()
			{
				let director_ws = Director::new(name, game_id.to_string(), game_addr);
				let resp = ws::start(director_ws, &req, stream);
				println!("{:?}", resp);
				return resp;
			}
		}
		"consumer" => {
			if let Some((game_addr, name)) = addr
				.send(handle_to_app::IsRegisteredPlayer {
					user_id: uuid.clone(),
					game_id: game_id.clone(),
				})
				.await
				.unwrap()
			{
				let consumer_ws = Consumer::new(name, game_id.to_string(), game_addr);
				let resp = ws::start(consumer_ws, &req, stream);
				println!("{:?}", resp);
				return resp;
			}
		}
		"producer" => {
			if let Some((game_addr, name)) = addr
				.send(handle_to_app::IsRegisteredPlayer {
					user_id: uuid.clone(),
					game_id: game_id.clone(),
				})
				.await
				.unwrap()
			{
				let producer_ws = Producer::new(name, game_id.to_string(), game_addr);
				let resp = ws::start(producer_ws, &req, stream);
				println!("{:?}", resp);
				return resp;
			}
		}
		"viewer" => {
			if let Some((game_addr, name)) = addr
				.send(handle_to_app::IsRegisteredViewer {
					user_id: uuid.clone(),
					game_id: game_id.clone(),
				})
				.await
				.unwrap()
			{
				let viewer_ws = Viewer::new(name, game_id.to_string(), game_addr);
				let resp = ws::start(viewer_ws, &req, stream);
				println!("{:?}", resp);
				return resp;
			}
		}
		_ => {
			return Ok(HttpResponse::build(http::StatusCode::OK).body("Invalid Viewtype"));
		}
	}
	Ok(HttpResponse::build(http::StatusCode::OK)
		.body("Failed: possible reasons are no cookies set or no corresponding uuid found"))
}

pub async fn handle_prep(req: HttpRequest) -> impl Responder {
	return format!(
		"{}\n{}\n{}",
		req.cookie("viewtype").unwrap().value(),
		req.cookie("game_id").unwrap().value(),
		req.cookie("uuid").unwrap().value()
	);
}
