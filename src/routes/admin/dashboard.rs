use actix_web::get;
use actix_web::HttpResponse;

#[get("admin/dashboard")]
pub async fn admin_dashboard() -> HttpResponse {
    HttpResponse::Ok().finish()
}
