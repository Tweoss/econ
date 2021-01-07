//* uses app.rs

use actix_files::Files;
use actix_web::{web, middleware, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix::{Actor, StreamHandler};
use actix_web_actors::ws;


/// Define HTTP actor
struct MyWs;

impl Actor for MyWs {
	type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
	fn handle(
		&mut self,
		msg: Result<ws::Message, ws::ProtocolError>,
		ctx: &mut Self::Context,
	) {
		match msg {
			// Ok(ws::Message::Text) => (),
			Ok(ws::Message::Text(text)) => {
				// let textt = "\"hehehehehehe: the\"".to_owned();
				ctx.text(text);
			},
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	std::env::set_var("RUST_LOG", "actix_web=debug");
	env_logger::init();
	HttpServer::new(|| {
		App::new()
			// Enable the logger.
			.wrap(middleware::Logger::default())
			// // We allow the visitor to see an index of the images at `/images`.
			// .service(Files::new("/images", "static/images/").show_files_listing())
			// fun
			.route("/hey", web::get().to(manual_hello))
			.route("/ws", web::get().to(handle_ws))
			// Serve a tree of static files at the web root and specify the index file.
			// Note that the root path should always be defined as the last item. The paths are
			// resolved in the order they are defined. If this would be placed before the `/images`
			// path then the service for the static images would never be reached.
			// .service(Files::new("/", "../client/root/static/").index_file("index.html"))
			.service(Files::new("/", "/home/runner/rust-server-wrapper/client/root/static/").index_file("index.html"))

	})
	// .bind("127.0.0.1:8080")?
	.bind("0.0.0.0:8080")?
	.run()
	.await
}