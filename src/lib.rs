use std::net::TcpListener;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder, dev::Server};

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World! \n"
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[get("/health_check")]
async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(index)
            .service(health_check)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
