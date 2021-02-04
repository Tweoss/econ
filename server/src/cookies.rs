use actix_web::{cookie, http, web, HttpRequest, HttpResponse};

use crate::application::app::AppState;
use crate::application::handle_to_app;

//* use in server.rs for auth
pub async fn set_cookies(req: HttpRequest) -> HttpResponse {
	println!("HIHIHIHIHIHIHIHIHIHI");
	let addr = req.app_data::<web::Data<actix::Addr<AppState>>>().unwrap();
	let game_id: String = req
		.headers()
		.get("game_id")
		.unwrap()
		.to_str()
		.unwrap()
		.to_string();
	let username = req.headers().get("username").unwrap().to_str().unwrap();
	let viewtype = req.headers().get("viewtype").unwrap().to_str().unwrap();
	println!(
		"Username: {:?}, Viewtype: {:?}, GameID: {:?}",
		username, viewtype, game_id
	);
	let mut id_cookie = cookie::Cookie::build("uuid", "")
		.same_site(cookie::SameSite::Strict)
		.secure(true)
		.max_age(time::Duration::hours(2)) // 2 hrs
		.finish();
	let mut viewtype_cookie = cookie::Cookie::build("viewtype", "")
		.same_site(cookie::SameSite::Strict)
		.secure(true)
		.max_age(time::Duration::hours(2)) // 2 hrs
		.finish();
	let name_cookie = cookie::Cookie::build("username", username)
		.same_site(cookie::SameSite::Strict)
		.secure(true)
		.max_age(time::Duration::hours(2)) // 2 hrs
		.finish();
	let game_id_cookie = cookie::Cookie::build("game_id", game_id.clone())
		.same_site(cookie::SameSite::Strict)
		.secure(true)
		.max_age(time::Duration::hours(2))
		.finish();
	let temp_uuid = uuid::Uuid::new_v4().to_hyphenated().to_string();
	if addr
		.send(handle_to_app::DoesGameExist {
			game_id: game_id.clone(),
		})
		.await
		.unwrap()
	{
		let is_open = addr
			.send(handle_to_app::IsGameOpen {
				game_id: game_id.clone(),
			})
			.await
			.unwrap();
		match viewtype {
			"player" => {
				if is_open {
					id_cookie.set_value("uuid string");
					viewtype_cookie.set_value(
						addr.send(handle_to_app::NewPlayer {
							user_id: temp_uuid,
							game_id,
							username: username.to_string(),
						})
						.await
						.unwrap(),
					);
					HttpResponse::build(http::StatusCode::OK)
						.cookie(id_cookie)
						.cookie(name_cookie)
						.cookie(viewtype_cookie)
						.cookie(game_id_cookie)
						.finish()
				} else {
					HttpResponse::Ok()
						.content_type("plain/text")
						.body("Game not open yet.")
				}
			}
			"director" => {
				let pswd = req
					.headers()
					.get("password")
					.unwrap()
					.to_str()
					.unwrap()
					.to_string();
				viewtype_cookie.set_value("director");
				if addr
					.send(handle_to_app::IsRightPswd { pswd })
					.await
					.unwrap()
				{
					// if is_open {
					addr.do_send(handle_to_app::NewDirector {
						user_id: temp_uuid,
						game_id,
						username: username.to_string(),
					});
					HttpResponse::build(http::StatusCode::OK)
						.cookie(id_cookie)
						.cookie(name_cookie)
						.cookie(viewtype_cookie)
						.cookie(game_id_cookie)
						.finish()
				// } else {
				// 	HttpResponse::build(http::StatusCode::OK)
				// 		.cookie(id_cookie)
				// 		.finish()
				// }
				} else {
					HttpResponse::Ok()
						.content_type("plain/text")
						.body("Invalid Password")
				}
			}
			_ => {
				println!("SMTH BAD HAPPENED");
				HttpResponse::Ok()
					.content_type("plain/text")
					.body("Unknown Viewing Type")
			}
		}
	} else if viewtype == "director" {
		viewtype_cookie.set_value("director");
		let pswd = req
			.headers()
			.get("password")
			.unwrap()
			.to_str()
			.unwrap()
			.to_string();
		if addr
			.send(handle_to_app::IsRightPswd { pswd })
			.await
			.unwrap()
		{
			addr.send(handle_to_app::NewGame {
				user_id: temp_uuid,
				game_id,
				username: username.to_string(),
			})
			.await
			.unwrap();
			HttpResponse::Ok()
				.cookie(id_cookie)
				.cookie(name_cookie)
				.cookie(viewtype_cookie)
				.cookie(game_id_cookie)
				.content_type("plain/text")
				.body("Successfully made game")
		} else {
			HttpResponse::Ok()
				.content_type("plain/text")
				.body("Invalid Password")
		}
	} else {
		HttpResponse::Ok()
			.content_type("plain/text")
			.body("No Game with that ID Found")
	}
}
