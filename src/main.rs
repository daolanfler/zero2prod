use std::net::TcpListener;

use env_logger::Env;
use sqlx::{Connection, PgConnection, PgPool};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // `init` does call `set_logger`
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect Postgres.");
    // succesfully connect to db

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
