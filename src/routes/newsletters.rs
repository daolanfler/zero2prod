use actix_web::{HttpResponse, post};


#[post("/newsletters")]
pub async fn publish_newsletter() -> HttpResponse {
    HttpResponse::Ok().finish()
}