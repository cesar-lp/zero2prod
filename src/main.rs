use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{configuration::get_configurations, startup};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configurations().expect("Failed to read configurations.");

    let connection_pool = PgPool::connect(&configuration.database.get_conection_string())
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{}", configuration.application.port);

    let listener = TcpListener::bind(address).expect("Failed to bind random port.");

    startup::run(listener, connection_pool)?.await
}
