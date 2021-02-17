use actix_files::NamedFile;
use actix_web::{cookie, http, web, HttpMessage, HttpRequest, HttpResponse};
use actix_web::{get, Responder};

use std::path::PathBuf;

use crate::application::app::AppState;
use crate::application::handle_to_app;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CookieInfo {
	username: String,
	viewtype: String,
	game_id: String,
	password: String,
}

pub async fn set_cookies(cookie_info: web::Json<CookieInfo>, req: HttpRequest) -> HttpResponse {
	println!("Called set_cookies with args: ");
	let (game_id, username, viewtype) = (
		cookie_info.game_id.clone(),
		cookie_info.username.clone(),
		cookie_info.viewtype.clone(),
	);
	println!(
		"Username: {:?}, Viewtype: {:?}, GameID: {:?}",
		username, viewtype, game_id
	);

	let addr = req
		.app_data::<web::Data<actix::Addr<AppState>>>()
		.unwrap();
	println!("\nThese are the cookies sent from client.");
	if let Ok(cookies) = req.cookies() {
		for cookie in cookies.to_owned() {
			print!("{}, ", cookie.to_string());
		}
	}
	println!();
	// ! SWITCH THIS REPL
	let mut id_cookie = cookie::Cookie::build("uuid", "")
		.same_site(cookie::SameSite::Strict)
		// .secure(true)
		.max_age(time::Duration::hours(2)) // 2 hrs
		.finish();
	let mut viewtype_cookie = cookie::Cookie::build("viewtype", "")
		.same_site(cookie::SameSite::Strict)
		// .secure(true)
		.max_age(time::Duration::hours(2)) // 2 hrs
		.finish();
	let name_cookie = cookie::Cookie::build("username", username.clone())
		.same_site(cookie::SameSite::Strict)
		// .secure(true)
		.max_age(time::Duration::hours(2)) // 2 hrs
		.finish();
	let game_id_cookie = cookie::Cookie::build("game_id", game_id.clone())
		.same_site(cookie::SameSite::Strict)
		// .secure(true)
		.max_age(time::Duration::hours(2))
		.finish();
	let temp_uuid = uuid::Uuid::new_v4().to_hyphenated().to_string();
	id_cookie.set_value(temp_uuid.to_string());
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
		match viewtype.as_ref() {
			"player" => {
				if is_open {
					viewtype_cookie.set_value(
						addr.send(handle_to_app::NewPlayer {
							user_id: temp_uuid,
							game_id,
							username: username.clone(),
						})
						.await
						.unwrap(),
					);
					HttpResponse::build(http::StatusCode::OK)
						.cookie(id_cookie)
						.cookie(name_cookie)
						.cookie(viewtype_cookie)
						.cookie(game_id_cookie)
						.content_type("plain/text")
						.body("Success")
				} else {
					HttpResponse::Ok()
						.content_type("plain/text")
						.body("Game not open yet.")
				}
			}
			"director" => {
				let pswd = cookie_info.password.clone();
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
		let pswd = cookie_info.password.clone();
		if addr
			.send(handle_to_app::IsRightPswd { pswd })
			.await
			.unwrap()
		{
			addr.send(handle_to_app::NewGame {
				user_id: temp_uuid.clone(),
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
				.body("Success")
		} else {
			HttpResponse::Ok()
				.content_type("plain/text")
				.body("Invalid Password")
		}
	} else {
		HttpResponse::Ok()
			.content_type("plain/text")
			.body("No Game with that ID Found")
		// HttpResponse::Found()
		// 	.header(http::header::LOCATION, "/viewer/123")
		// 	.finish()
	}
}

// ! PLAN TO MAKE DIFFERENT HANDLERS
#[get("/{play_view_direct}/{type}/{gameid:\\d*}/{filename}.{ext}")]
// type for authenticated directors or viewers are director and viewer respectively
async fn get_html(req: HttpRequest) -> impl Responder {
	// http://localhost:8080/play/producer/gameid/index.html
	println!("Received request for Files");
	let mut prepath = "../client/".to_owned();
	// let prepath = "../client/root/static/";
	let filename = req.match_info().get("filename").unwrap();
	let ext = req.match_info().get("ext").unwrap();
	let _play_view_direct = req.match_info().get("play_view_direct").unwrap();
	let url_viewtype = req.match_info().get("type").unwrap();
	let url_game_id = req.match_info().get("gameid").unwrap();
	if let (Some(view_type), Some(game_id)) = (req.cookie("viewtype"), req.cookie("game_id")) {
		if view_type.value() == url_viewtype && game_id.value() == url_game_id {
			prepath = match view_type.value() {
				"viewer" => prepath + "viewer/",
				"producer" => prepath + "producer/",
				"consumer" => prepath + "consumer/",
				"director" => prepath + "director_auth/",
				// Ok(NamedFile::open(
				// 	(prepath + filename + "." + ext).parse::<PathBuf>().unwrap(),
				// ));
				_ => return Ok(NamedFile::open("../client/404/static/index.html")),
			};
			prepath += "static/";
			let full_path = (*prepath).to_owned() + filename + "." + ext;
			match ext {
				"html" | "js" | "css" | "wasm" => {
					// if let Some(view_type) = req.cookie("viewtype") {
					println!("HI: {cat}", cat = full_path);
					return Ok(NamedFile::open(
						(prepath + filename + "." + ext).parse::<PathBuf>().unwrap(),
					));
				}
				_ => (),
			}
		}
	}

	Err(actix_web::error::ErrorUnauthorized("Not Authorized"))
	// println!(
	// 	"{prepath}\n{file}\n{ext}",
	// 	prepath = prepath,
	// 	file = filename,
	// 	ext = ext
	// );
	// let prepath: PathBuf = "../client/producer/".parse().unwrap();
	// Ok(NamedFile::open(path)?)
}

pub async fn redirect(req: HttpRequest) -> impl Responder {
	if let Some(game_id) = req.cookie("game_id") {
		if let Some(viewtype) = req.cookie("viewtype") {
			match viewtype.value() {
				"director" => {
					// return HttpResponse::build(http::StatusCode::OK)
					// 	.body("HI")
					return HttpResponse::build(http::StatusCode::FOUND)
						.header(
							http::header::LOCATION,
							format!("direct/{}/index.html", game_id.value()),
						)
						.finish();
				}
				"consumer" => {
					return HttpResponse::build(http::StatusCode::FOUND)
						.header(
							http::header::LOCATION,
							format!("play/consumer/{}/index.html", game_id.value()),
						)
						.finish()
				}
				"producer" => {
					return HttpResponse::build(http::StatusCode::FOUND)
						.header(
							http::header::LOCATION,
							format!("play/producer/{}/index.html", game_id.value()),
						)
						.finish()
				}
				_ => (),
			}
		}
	}

	HttpResponse::build(http::StatusCode::FOUND)
		.header(http::header::LOCATION, "/login")
		.finish()
	// if let cookies = req.cookies() {
	// 	cookies = ();
	// } else {
	// 	HttpResponse::build(http::StatusCode::PERMANENT_REDIRECT)
	// 		.header(http::header::LOCATION, "/viewer/123")
	// 		.finish()
	// }
}
