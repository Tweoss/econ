use actix_web::HttpMessage;
use actix_web::{http, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::application::app::AppState;
use crate::application::participants::player::MyWs;

pub async fn handle_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
	// println!("called handle_ws");
	if let (Some(viewtype), Some(game_id), Some(uuid)) = (req.cookie("viewtype"), req.cookie("game_id"), req.cookie("uuid")) {
		let addr = req.app_data::<web::Data<actix::Addr<AppState>>>().unwrap();
		// addr.send(crate::application::handle_to_app::IsGameOpen{game_id: "HI".to_string()});

		match viewtype.value() {
			"director" => {
				let resp = ws::start(MyWs::new(uuid.to_string(), game_id.to_string(), addr), &req, stream);
				println!("{:?}", resp);
				resp
			}
			"consumer" => {
				let resp = ws::start(MyWs::new(uuid.to_string(), game_id.to_string(), addr), &req, stream);
				println!("{:?}", resp);
				resp
			}
			"producer" => {
				let resp = ws::start(MyWs::new(uuid.to_string(), game_id.to_string(), addr), &req, stream);
				println!("{:?}", resp);
				resp
			}
			"viewer" => {
				let resp = ws::start(MyWs::new(uuid.to_string(), game_id.to_string(), addr), &req, stream);
				println!("{:?}", resp);
				resp
			}
			_ => Ok(HttpResponse::build(http::StatusCode::OK).body("Invalid Viewtype")),
		}
	} else {
		Ok(HttpResponse::build(http::StatusCode::OK).body("No Cookie Set"))
	}
}
