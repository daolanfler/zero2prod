use std::net::TcpListener;

use actix_web::{dev::Server, get, web, App, HttpRequest, HttpResponse, HttpServer, Responder, post};

#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

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

#[post("/subscriptions")]
async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(index)
            .service(health_check)
            .service(subscribe)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
