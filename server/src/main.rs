//* uses app.rs

use actix::{Actor};
use actix_files::Files;
use actix_files::NamedFile;
use actix_web::{middleware, web, App, HttpRequest, HttpServer};

// use uuid::Uuid;

mod application;
use crate::application::app::AppState;

mod html_handlers;
use html_handlers::{get_html, redirect, set_cookies};

mod ws_handler;
use ws_handler::{handle_ws};

// /// Define HTTP actor
// struct MyWs;

// impl Actor for MyWs {
// 	type Context = ws::WebsocketContext<Self>;
// }

// /// Handler for ws::Message message
// impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
// 	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
// 		match msg {
// 			// Ok(ws::Message::Text) => (),
// 			Ok(ws::Message::Text(text)) => {
// 				ctx.text(text);
// 			}
// 			Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
// 			_ => (),
// 		}
// 	}
// }


async fn index_404(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    // let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open("../client/404/static/index.html")?)
}



// #[get("/director/{filename}.{ext}")]

// #[get("/login/{filename}.{ext}")]

// #[get("/play/{producer}/{gameid}/{filename}.{ext}")]

//* set the auth cookies MAKE SURE TO CHECK COOKIES AT LOGIN AND AT PLAY URL

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
	HttpServer::new(|| {
		let path: String = "../client/".to_owned();
		// let path: String = "/home/runner/rust-server-wrapper/client/".to_owned();

		let app_addr = AppState::new().start();
		App::new()
			.wrap(middleware::Logger::default())
			.data(app_addr)
			// .route("/ws", web::get().to(handle_ws))
			.route("/cookies", web::post().to(set_cookies))
			.route("/redirect", web::get().to(redirect))
			.service(
				Files::new("/director/index", path.clone() + "/director_auth/static/")
					.index_file("index.html"),
			)
			.service(
				Files::new("/director/{gameid}", path.clone() + "director_auth/static/")
					.index_file("index.html"),
			)
			.service(
				Files::new("/viewer/{gameid}", path.clone() + "viewer/static/")
					.index_file("index.html"),
			)
			.route("/ws", web::get().to(handle_ws))
			.service(get_html)
			.service(Files::new("/login",  path + "login/static/").index_file("index.html"))
			.default_service(web::get().to(index_404))
	})
	// ! SWITCH THIS REPL
	// .bind("127.0.0.1:8080")?
	.bind("0.0.0.0:8080")?
	.run()
	.await
}
