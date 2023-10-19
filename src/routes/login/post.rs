use actix_web::http::header::LOCATION;
use actix_web::{post, HttpResponse};

#[post("/login")]
pub async fn login() -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish()
}
