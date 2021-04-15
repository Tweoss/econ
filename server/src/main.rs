//* uses app.rs

use actix::Actor;
use actix_files::Files;
use actix_files::NamedFile;
use actix_web::{middleware, web, App, HttpRequest, HttpServer};

// use uuid::Uuid;

mod application;
use crate::application::app::AppState;

mod html_handlers;
use html_handlers::{assets, get_html, inline, redirect, set_cookies};

mod ws_handler;
use ws_handler::{handle_prep, handle_ws};

mod handle_to_app;

async fn index_404(_req: HttpRequest) -> actix_web::Result<NamedFile> {
	Ok(NamedFile::open("../client/404/static/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let _guard = sentry::init((
		"https://0bccce907caa48179289796c6d3d1b6f@o440162.ingest.sentry.io/5604404",
		sentry::ClientOptions {
			release: sentry::release_name!(),
			..Default::default()
		},
	));

	std::env::set_var("RUST_LOG", "actix_web=debug");
	env_logger::init();
	println!("Starting server.");
	let app_addr = AppState::new().start();
	HttpServer::new(move || {
		let path: String = "../client/".to_owned();

		App::new()
			.wrap(middleware::Logger::default())
			.data(app_addr.clone())
			.route("/cookies", web::post().to(set_cookies))
			.route("/redirect", web::get().to(redirect))
			.route("/ws/{viewtype}/{game_id}/{uuid}", web::get().to(handle_ws))
			.route("/wsprep", web::post().to(handle_prep))
			.service(get_html)
			.service(assets)
			.service(inline)
			.service(
				Files::new("/director_login", path.clone() + "director_login/static/")
					.index_file("index.html"),
			)
			.service(Files::new("/login", path.clone() + "login/static/").index_file("index.html"))
			.service(Files::new("/view", path + "viewer_login/static/").index_file("index.html"))
			.default_service(web::get().to(index_404))
	})
	.bind("0.0.0.0:8080")?
	.run()
	.await
}
