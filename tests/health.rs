use std::net::TcpListener;

use reqwest::StatusCode;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configuration::{get_configurations, DatabaseSettings};

struct TestApp {
    address: String,
    db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_201_for_valid_body() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    let saved = sqlx::query!("SELECT name, email FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_body() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-url-encoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not fail with 400 when the payload was {}.",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp {
    let mut configuration = get_configurations().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();

    let connection_pool = configure_db(&configuration.database).await;

    let server = zero2prod::startup::run(listener, connection_pool.clone())
        .expect("Failed to bind address.");

    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}

async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.get_conection_string_without_db().as_str())
        .await
        .expect("Failed to connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.get_conection_string())
        .await
        .expect("Failed to establish Postgres connection pool");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database.");

    connection_pool
}
