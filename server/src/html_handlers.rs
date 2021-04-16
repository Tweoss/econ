use actix_files::NamedFile;
use actix_web::{cookie, http, web, HttpMessage, HttpRequest, HttpResponse};
use actix_web::{get, Responder};

use std::path::PathBuf;

use crate::application::app::AppState;

use crate::handle_to_app;

use serde::Deserialize;

const DEPLOY_OR_STATIC: &str = "static";

#[derive(Deserialize)]
pub struct CookieInfo {
	username: String,
	viewtype: String,
	game_id: String,
	password: String,
}

pub async fn set_cookies(cookie_info: web::Json<CookieInfo>, req: HttpRequest) -> HttpResponse {
	let (game_id, username, viewtype) = (
		cookie_info.game_id.clone(),
		cookie_info.username.clone(),
		cookie_info.viewtype.clone(),
	);
	println!(
		"Called set_cookies with args: \n
		Username: {:?}, Viewtype: {:?}, GameID: {:?}",
		username, viewtype, game_id
	);
	if !game_id.chars().any(|c| c.is_ascii_digit()) {
		return HttpResponse::Ok()
			.content_type("plain/text")
			.body("Invalid Game ID");
	}
	let addr = req.app_data::<web::Data<actix::Addr<AppState>>>().unwrap();

	//* make sure isn't main director
	if let (Some(uuid), Some(viewtype), Some(game_id)) = (
		req.cookie("uuid"),
		req.cookie("viewtype"),
		req.cookie("game_id"),
	) {
		if viewtype.value() == "director"
			&& addr
				.send(handle_to_app::IsMainDirector {
					game_id: game_id.value().to_string(),
					user_id: uuid.value().to_string(),
				})
				.await
				.unwrap()
		{
			return HttpResponse::Ok()
  					.content_type("plain/text")
  					.body(format!("Cannot join a game while being a main director. Go to /direct/director/{}/index.html", game_id.value()));
		}
	}
	// ! SWITCH THIS REPL
	let mut id_cookie = cookie::Cookie::build("uuid", "")
		.same_site(cookie::SameSite::Strict)
		// .secure(true)
		.max_age(time::Duration::hours(2)) // 2 hrs
		.finish();
	let mut viewtype_cookie = cookie::Cookie::build("viewtype", "")
		.same_site(cookie::SameSite::Strict)
		// .secure(true)
		.max_age(time::Duration::hours(2))
		.finish();
	let name_cookie = cookie::Cookie::build("username", username.clone())
		.same_site(cookie::SameSite::Strict)
		// .secure(true)
		.max_age(time::Duration::hours(2))
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
					let player_type = addr
						.send(handle_to_app::NewPlayer {
							user_id: temp_uuid,
							game_id,
							username: username.clone(),
						})
						.await
						.unwrap();
					if player_type == "Name taken" {
						HttpResponse::Ok()
							.content_type("plain/text")
							.body("Name taken")
					} else {
						viewtype_cookie.set_value(player_type);
						HttpResponse::build(http::StatusCode::OK)
							.cookie(id_cookie)
							.cookie(name_cookie)
							.cookie(viewtype_cookie)
							.cookie(game_id_cookie)
							.content_type("plain/text")
							.body("Success")
					}
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
						.content_type("plain/text")
						.body("Success")
				} else {
					HttpResponse::Ok()
						.content_type("plain/text")
						.body("Invalid Password")
				}
			}
			"viewer" => {
				viewtype_cookie.set_value("viewer");
				if addr
					.send(handle_to_app::NewViewer {
						user_id: temp_uuid,
						game_id,
						username: username.clone(),
					})
					.await
					.unwrap()
				{
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
						.body("Name taken")
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
	}
}

#[get("/{play_view_direct}/{type}/{gameid:\\d*}/{filename}.{ext}")]
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
		let addr = req.app_data::<web::Data<actix::Addr<AppState>>>().unwrap();
		if view_type.value() == url_viewtype
			&& game_id.value() == url_game_id
			&& addr
				.send(handle_to_app::DoesGameExist {
					game_id: game_id.value().to_string(),
				})
				.await
				.unwrap()
		{
			prepath = match view_type.value() {
				"viewer" => prepath + "viewer/",
				"producer" => prepath + "producer/",
				"consumer" => prepath + "consumer/",
				"director" => prepath + "director_auth/",
				_ => {
					return Ok(NamedFile::open(format!(
						"../client/404/{}/index.html",
						DEPLOY_OR_STATIC
					)))
				}
			};
			prepath += DEPLOY_OR_STATIC;
			prepath += "/";
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

	Err(actix_web::error::ErrorUnauthorized("Game does not exist."))
}

pub async fn redirect(req: HttpRequest) -> impl Responder {
	if let Some(game_id) = req.cookie("game_id") {
		if let Some(viewtype) = req.cookie("viewtype") {
			match viewtype.value() {
				"director" => {
					return HttpResponse::build(http::StatusCode::FOUND)
						.header(
							http::header::LOCATION,
							format!("direct/director/{}/index.html", game_id.value()),
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
				"viewer" => {
					return HttpResponse::build(http::StatusCode::FOUND)
						.header(
							http::header::LOCATION,
							format!("view/viewer/{}/index.html", game_id.value()),
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
}

#[get("/{play_view_direct}/{type}/{gameid:\\d*}/assets/{rest_of_path:.*}")]
async fn assets(req: HttpRequest) -> impl Responder {
	let mut prepath = "../client/".to_owned();
	let url_viewtype = req.match_info().get("type").unwrap();
	let path = req.match_info().get("rest_of_path").unwrap();
	prepath = match url_viewtype {
		"viewer" => prepath + "viewer/",
		"producer" => prepath + "producer/",
		"consumer" => prepath + "consumer/",
		"director" => prepath + "director_auth/",
		_ => {
			let failed: actix_files::NamedFile =
				NamedFile::open(format!("../client/404/{}/index.html", DEPLOY_OR_STATIC)).unwrap();
			return failed;
		}
	};
	prepath += DEPLOY_OR_STATIC;
	prepath += "/assets/";
	prepath += path;
	println!("{}", prepath);
	NamedFile::open(prepath).unwrap()
}

#[get("/{play_view_direct}/{type}/{gameid:\\d*}/snippets/{folder}/{rest_of_path:.*}.js")]
async fn inline(req: HttpRequest) -> impl Responder {
	let mut prepath = "../client/".to_owned();
	let url_viewtype = req.match_info().get("type").unwrap();
	let folder = req.match_info().get("folder").unwrap();
	let path = req.match_info().get("rest_of_path").unwrap();
	prepath = match url_viewtype {
		"viewer" => prepath + "viewer/",
		"producer" => prepath + "producer/",
		"consumer" => prepath + "consumer/",
		"director" => prepath + "director_auth/",
		_ => {
			let failed: actix_files::NamedFile =
				NamedFile::open(format!("../client/404/{}/index.html", DEPLOY_OR_STATIC)).unwrap();
			return failed;
		}
	};
	prepath += DEPLOY_OR_STATIC;
	prepath += "/snippets/";
	prepath += folder;
	prepath += "/";
	prepath += path;
	prepath += ".js";
	println!("{}", prepath);
	NamedFile::open(prepath).unwrap()
}
