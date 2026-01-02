use crate::theme_service::ThemeService;
use actix_web::{HttpResponse, Result, get, web};

#[get("/health")]
pub async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "message": "Service is healthy"
    })))
}

/// Serve theme CSS (server proxies external API)
pub async fn serve_theme_css(
    theme_service: web::Data<std::sync::Arc<ThemeService>>,
) -> Result<HttpResponse> {
    let css = theme_service.get_css().await;

    Ok(HttpResponse::Ok()
        .content_type("text/css")
        .append_header(("Cache-Control", "public, max-age=300")) // 5 minute cache
        .body(css))
}

/// Force refresh theme cache (useful for debugging)
pub async fn refresh_theme_cache(
    theme_service: web::Data<std::sync::Arc<ThemeService>>,
) -> Result<HttpResponse> {
    theme_service.refresh_cache().await;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(serde_json::json!({
            "status": "ok",
            "message": "Theme cache refreshed"
        })))
}
