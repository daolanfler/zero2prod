use actix_web::{post, web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
pub async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
