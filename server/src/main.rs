//* uses app.rs

use actix::{Actor, StreamHandler};
use actix_files::{Files, NamedFile};
use actix_web::{
	cookie, get, http, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
	Responder,
};
use actix_web_actors::ws;
use std::path::PathBuf;
// use uuid::Uuid;

mod application;

use application::app::AppState;
use application::app_messages;

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
				// let textt = "\"hehehehehehe: the\"".to_owned();
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

async fn manual_hello() -> impl Responder {
	HttpResponse::Ok().body("Hey there!")
}

//todo UNNECESSARY??
#[get("/play/producer/{gameid}/")]
async fn redirect() -> impl Responder {
	//* ABSOLUTE URL FOR SOME REASON
	HttpResponse::PermanentRedirect()
		.header("LOCATION", "../index.html")
		.finish()
}
// #[get("/director/producer/{gameid}/{filename}.{ext}")]
// async fn redirect() -> impl Responder {
// 	HttpResponse::PermanentRedirect().header("LOCATION","/index.html")
// }
// #[get("/play/producer/{gameid}/{filename}.{ext}")]
// async fn redirect() -> impl Responder {
// 	HttpResponse::PermanentRedirect().header("LOCATION","/index.html")
// }
// #[get("/director/{filename}.{ext}")]

// #[get("/login/{filename}.{ext}")]

#[get("/play/{producer}/{gameid}/{filename}.{ext}")]
//* #[get("/{play_view_direct}/{type}/{gameid}/{filename}.{ext}")]
// type for authenticated directors or viewers are direct and view respectively
// async fn html(req: HttpRequest) -> impl Responder {
async fn html_produce(req: HttpRequest) -> impl Responder {
	// http://localhost:8080/play/producer/gameid/index.html
	println!("Received request for Files");
	// let prepath = "../client/producer/";
	let prepath = "../client/root/static/";
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
// #[get("/plal")]
// // #[get("/play/producer/{gameid}/index.html")]
// async fn html_produce_html(_req: HttpRequest) -> Result<NamedFile, Error> {
// 	let path: PathBuf = "../client/consumer/index.html".parse().unwrap();
// 	Files::new("/viewer/{gameid}", "viewer/static").index_file("index.html");
// 	Ok(NamedFile::open(path)?)
// }
// #[get("/play/consumer/{gameid}:.*")]
// async fn html_consume(_req: HttpRequest) -> Result<NamedFile, Error> {
// 	let path: PathBuf = "../client/consumer/index.html".parse().unwrap();
// 	Files::new("/viewer/{gameid}", "viewer/static/").index_file("index.html");
// 	// Files::new("/path", ".")
// 	Files::new("/viewer/{gameid}", "viewer/static/").index_file("index.html");
// 	Ok(NamedFile::open(path)?)
// }

// async fn data_produce(req: HttpRequest) -> impl Responder {
// 	let name = req.match_info().get("name").unwrap_or("World");
// 	format!("Hello {}!", &name)
// }
// async fn data_consume(req: HttpRequest) -> impl Responder {
// 	let name = req.match_info().get("name").unwrap_or("World");
// 	format!("Hello {}!", &name)
// }

//* set the auth cookies MAKE SURE TO CHECK COOKIES AT LOGIN AND AT PLAY URL
async fn set_cookies(req: HttpRequest, data: web::Data<actix::Addr<AppState>>) -> HttpResponse {
	println!("HIHIHIHIHIHIHIHIHIHI");
	let opt_t = req.app_data::<web::Data<actix::Addr<AppState>>>();

	let game_id: String = req
		.headers()
		.get("game_id")
		.unwrap()
		.to_str()
		.unwrap()
		.to_string();
	let _username = req.headers().get("username").unwrap().to_str().unwrap();
	let viewtype = req.headers().get("viewtype").unwrap().to_str().unwrap();
	let mut cookie = cookie::Cookie::build("uuid", "")
		.same_site(cookie::SameSite::Strict)
		.secure(true)
		.max_age(time::Duration::hours(2)) // 2 hrs
		.finish();

	// ! need to implement game_exists msg and handler
	// //* player or director
	if opt_t
		.clone().unwrap()
		.send(app_messages::DoesGameExist { game_id })
		.await
		.unwrap()
	{
		match viewtype {
			"player" => {
				cookie.set_value("uuid string");
				()
			}
			"director" => {
				let pswd = req.headers().get("password").unwrap().to_str().unwrap();
				()
			}
			_ => (),
		}
	} else {
		// if
	}
	let res = HttpResponse::build(http::StatusCode::OK)
		.cookie(cookie)
		.finish();
	res
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	std::env::set_var("RUST_LOG", "actix_web=debug");
	env_logger::init();
	HttpServer::new(|| {
		let path: String = "../client/".to_owned();
		// let path: String = "/home/runner/rust-server-wrapper/client/".to_owned();

		let app_adr = AppState::new().start();
		App::new()
			// Enable the logger.
			.wrap(middleware::Logger::default())
			.data(app_adr)
			// fun
			.route("/hey", web::get().to(manual_hello))
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
