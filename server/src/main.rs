//* uses app.rs

use actix::{Actor, StreamHandler};
use actix_files::{Files, NamedFile};
use actix_web::{
	get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
	Responder,
};
use actix_web_actors::ws;
use std::path::PathBuf;
// use uuid::Uuid;

mod application;

use application::app::AppState;

mod cookies;
use cookies::set_cookies;


/// Define HTTP actor
struct MyWs;

impl Actor for MyWs {
	type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		match msg {
			// Ok(ws::Message::Text) => (),
			Ok(ws::Message::Text(text)) => {
				ctx.text(text);
			}
			Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
			_ => (),
		}
	}
}

async fn handle_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
	// println!("called handle_ws");
	let resp = ws::start(MyWs {}, &req, stream);
	println!("{:?}", resp);
	resp
}

//todo UNNECESSARY??
#[get("/play/producer/{gameid}/")]
async fn redirect() -> impl Responder {
	//* ABSOLUTE URL FOR SOME REASON
	HttpResponse::PermanentRedirect()
		.header("LOCATION", "../index.html")
		.finish()
}

// #[get("/director/{filename}.{ext}")]

// #[get("/login/{filename}.{ext}")]

#[get("/play/{producer}/{gameid}/{filename}.{ext}")]
//* #[get("/{play_view_direct}/{type}/{gameid}/{filename}.{ext}")]
// type for authenticated directors or viewers are direct and view respectively
async fn html_produce(req: HttpRequest) -> impl Responder {
	// http://localhost:8080/play/producer/gameid/index.html
	println!("Received request for Files");
	// let prepath = "../client/producer/";
	let prepath = "../client/root/static/";
	// let mut path = "../client/";
	let filename: &str = req.match_info().get("filename").unwrap();
	let ext: &str = req.match_info().get("ext").unwrap();
	println!(
		"{prepath}\n{file}\n{ext}",
		prepath = prepath,
		file = filename,
		ext = ext
	);
	match ext {
		"html" | "js" => {
			println!("got a request");
			if true {
				let temp = (*prepath).to_owned() + filename + "." + ext;
				println!("HI: {cat}", cat = temp);
				Ok(NamedFile::open(
					(prepath.to_owned() + filename + "." + ext)
						.parse::<PathBuf>()
						.unwrap(),
				))
			} else {
				Err(actix_web::error::ErrorUnauthorized("Not Authorized"))
			}
		}
		_ => {
			Err(actix_web::error::ErrorUnauthorized("Not Authorized"))
			// 	println!("HILOOK AT ME");
			// let temp = (*prepath).to_owned() + filename + "." + ext;
			// Ok(NamedFile::open((prepath.to_owned() + filename + "." + ext).parse::<PathBuf>().unwrap()))
		}
	}
	// let prepath: PathBuf = "../client/producer/".parse().unwrap();
	// Ok(NamedFile::open(path)?)
}

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
			.route("/ws", web::get().to(handle_ws))
			.route("/cookies", web::get().to(set_cookies))
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
			.service(html_produce)
			.service(redirect)
			// .route("/data/producer", web::get().to(data_produce))
			// .route("/data/consumer", web::get().to(data_consume))
			.service(Files::new("/", path + "root/static/").index_file("index.html"))
	})
	.bind("127.0.0.1:8080")?
	// .bind("0.0.0.0:8080")?
	.run()
	.await
}
