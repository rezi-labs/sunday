use actix_web::{HttpResponse, Result, get};

#[get("/health")]
pub async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "Service is healthy"
    })))
}
