use crate::config::Server;
use crate::db::theme::ThemeCssGenerator;
use actix_files::Files;
use actix_web::{HttpResponse, Result as AwResult, get, web};

#[get("/generated/assets/themes.css")]
pub async fn dynamic_themes_css(server: web::Data<Server>) -> AwResult<HttpResponse> {
    match ThemeCssGenerator::get_or_generate_css(&server.pg_pool).await {
        Ok(css) => Ok(HttpResponse::Ok()
            .content_type("text/css; charset=utf-8")
            .insert_header(("Cache-Control", "public, max-age=300")) // Cache for 5 minutes
            .body(css)),
        Err(e) => {
            log::error!("Failed to generate themes.css: {}", e);
            Ok(HttpResponse::InternalServerError()
                .content_type("text/css")
                .body("/* Error generating themes */"))
        }
    }
}

pub fn scope() -> actix_web::Scope {
    web::scope("")
        .service(dynamic_themes_css)
        .service(
            web::scope("/assets")
                .service(Files::new("", "assets").show_files_listing())
        )
}
