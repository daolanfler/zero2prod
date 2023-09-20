use actix_web::{get, HttpRequest, HttpResponse, Responder};

#[get("/health_check")]
async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}
