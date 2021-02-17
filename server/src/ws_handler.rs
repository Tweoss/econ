use actix_web::HttpMessage;
use actix_web::Responder;
use actix_web::{http, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::application::app::AppState;
use crate::application::participants::director::Director;

pub async fn handle_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
	println!("called handle_ws");
	let viewtype: String = req.match_info().get("viewtype").unwrap().parse().unwrap();
	let game_id: String = req.match_info().get("game_id").unwrap().parse().unwrap();
	let uuid: String = req.match_info().get("uuid").unwrap().parse().unwrap();
	// if let (Some(viewtype), Some(game_id), Some(uuid)) = (
	// 	req.cookie("viewtype"),
	// 	req.cookie("game_id"),
	// 	req.cookie("uuid"),
	// ) {
	let addr = req.app_data::<web::Data<actix::Addr<AppState>>>().unwrap();
	// addr.send(crate::application::handle_to_app::IsGameOpen{game_id: "HI".to_string()});

	println!("Had some cookies");
	match viewtype.as_ref() {
		"director" => {
			println!("Asking for Director");

			if let Some(director_ws) =
				Director::new(uuid.to_string(), game_id.to_string(), addr.clone()).await
			{
				let resp = ws::start(director_ws, &req, stream);
				println!("{:?}", resp);
				return resp;
			}
		}
		"consumer" => {
			// let resp = ws::start(Director::new(uuid.to_string(), game_id.to_string(), addr), &req, stream);
			// println!("{:?}", resp);
			// resp;
		}
		"producer" => {
			// let resp = ws::start(Director::new(uuid.to_string(), game_id.to_string(), addr), &req, stream);
			// println!("{:?}", resp);
			// resp;
		}
		"viewer" => {
			// let resp = ws::start(Director::new(uuid.to_string(), game_id.to_string(), addr), &req, stream);
			// println!("{:?}", resp);
			// resp;
		}
		_ => {
			return Ok(HttpResponse::build(http::StatusCode::OK).body("Invalid Viewtype"));
		}
	}
	// }
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
