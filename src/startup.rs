use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgConnection;
use std::net::TcpListener;

pub fn run(listener: TcpListener, connection: PgConnection) -> Result<Server, std::io::Error> {
    // web::Data wraps our connection in an Atomic Reference Counted pointer (ARC)
    let connection = web::Data::new(connection);

    let server = HttpServer::new(move || {
        App::new()
            .service(health_check)
            .service(subscribe)
            // Get a pointer copy and attach it to the application state
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
