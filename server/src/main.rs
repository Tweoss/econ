use actix_files::Files;
// use actix_web::{web, middleware, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
// use actix::{Actor, StreamHandler};
// use actix_web_actors::ws;


// /// Define HTTP actor
// struct MyWs;

// impl Actor for MyWs {
// 	type Context = ws::WebsocketContext<Self>;
// }

// /// Handler for ws::Message message
// impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
// 	fn handle(
// 		&mut self,
// 		msg: Result<ws::Message, ws::ProtocolError>,
// 		ctx: &mut Self::Context,
// 	) {
// 		match msg {
// 			// Ok(ws::Message::Text) => (),
// 			Ok(ws::Message::Text(text)) => {
// 				ctx.text(text)
// 			},
// 			Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
// 			_ => (),
// 		}
// 	}
// }


// async fn handle_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
// 	println!("HIIII");
// 	let resp = ws::start(MyWs {}, &req, stream);
// 	println!("{:?}", resp);
// 	println!("{:?}", req);
// 	resp
// }


// async fn manual_hello() -> impl Responder {
// 	println!("HIIIIa");
// 	HttpResponse::Ok().body("Hey there!")
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
// 	std::env::set_var("RUST_LOG", "actix_web=debug");
// 	env_logger::init();

// 	HttpServer::new(|| {
// 		App::new()
// 			// Enable the logger.
// 			.wrap(middleware::Logger::default())
// 			// We allow the visitor to see an index of the images at `/images`.
// 			.service(Files::new("/images", "static/images/").show_files_listing())
// 			// fun
// 			.route("/hey", web::get().to(manual_hello))
// 			// Serve a tree of static files at the web root and specify the index file.
// 			// Note that the root path should always be defined as the last item. The paths are
// 			// resolved in the order they are defined. If this would be placed before the `/images`
// 			// path then the service for the static images would never be reached.
// 			.service(Files::new("/", "../client/root/static/").index_file("index.html"))
// 			// .service(web::resource("/ws").to(handle_ws))
// 			// HttpServer::new(|| App::new().route("/ws/", web::get().to(index)))

// 			.route("/ws/", web::get().to(handle_ws))
// 	})
// 	.bind("127.0.0.1:8080")?
// 	.run()
// 	.await
// }


use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
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
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs {}, &req, stream);
    println!("{:?}", resp);
    println!("2 hi");
    println!("{:?}", req);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	std::env::set_var("RUST_LOG", "actix_web=info");
	env_logger::init();
    println!("1 hi");
	HttpServer::new(|| {App::new().route("/ws", web::get().to(index))
	.service(Files::new("/", "../client/root/static/").index_file("index.html"))
	// .resource("/ws", |r| {
	// 	r.method(http::Method::GET).f(|r| ws::start(r, WebSocket::new()))
	// })	
})
        .bind("127.0.0.1:8080")?
        .run()
        .await
}