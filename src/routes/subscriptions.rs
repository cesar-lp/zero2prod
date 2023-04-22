use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let sql_result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, name, email, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.name,
        form.email,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await;

    return if sql_result.is_err() {
        println!("Failed to execute query: {}", sql_result.unwrap_err());
        HttpResponse::InternalServerError().finish()
    } else {
        HttpResponse::Created().finish()
    };
}
