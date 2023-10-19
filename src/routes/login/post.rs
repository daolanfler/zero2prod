use actix_web::http::header::LOCATION;
use actix_web::{post, HttpResponse, web};
use secrecy::Secret;

#[post("/login")]
pub async fn login(formData: web::Data<FormData>) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish()
}

#[derive(serde::Deserialize)]
struct FormData {
    username: String,
    password: Secret<String>,
}