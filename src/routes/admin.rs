use crate::config::{Server, DatabaseClient};
use crate::db;
use crate::routes::get_user;
use crate::view;
use crate::user::User;
use actix_web::{HttpRequest, HttpResponse, Result, error::ErrorInternalServerError, web};
use serde::Deserialize;
use validator::Validate;

pub async fn dashboard(
    req: HttpRequest,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    let user = get_user(req.clone());
    if let Some(user) = user.as_ref() {
        if !user.is_admin() {
            return Ok(HttpResponse::Forbidden()
                .content_type("text/html")
                .body("Access denied"));
        }
    } else {
        return Ok(HttpResponse::Forbidden()
            .content_type("text/html")
            .body("Access denied"));
    }

    // Get all users for the dashboard
    let users = match db::user::list_all(&db_client, None).await {
        Ok(users) => users,
        Err(e) => {
            log::error!("Failed to load users: {}", e);
            return Err(ErrorInternalServerError("Failed to load users"));
        }
    };

    let html = view::admin::dashboard(users, user.as_ref(), None);
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

#[derive(Deserialize, Validate)]
pub struct ThemeSettingsForm {
    pub default_light_theme: String,
    pub default_dark_theme: String,
}

pub async fn theme_settings_page(
    req: HttpRequest,
    server: web::Data<Server>,
) -> Result<HttpResponse> {
    let user = get_user(req.clone());
    if let Some(user) = user.as_ref() {
        if !user.is_admin() {
            return Ok(HttpResponse::Forbidden()
                .content_type("text/html")
                .body("Access denied"));
        }
    } else {
        return Ok(HttpResponse::Forbidden()
            .content_type("text/html")
            .body("Access denied"));
    }

    // Get all available themes
    let themes = match db::theme::Theme::get_all(&server.pg_pool).await {
        Ok(themes) => themes,
        Err(e) => {
            log::error!("Failed to load themes: {}", e);
            return Err(ErrorInternalServerError("Failed to load themes"));
        }
    };

    // Get current system theme settings
    let (light_theme, dark_theme) = match tokio::try_join!(
        db::theme::SystemThemeSettings::get_default_light_theme(&server.pg_pool),
        db::theme::SystemThemeSettings::get_default_dark_theme(&server.pg_pool)
    ) {
        Ok((light, dark)) => (light, dark),
        Err(e) => {
            log::error!("Failed to load theme settings: {}", e);
            return Err(ErrorInternalServerError("Failed to load theme settings"));
        }
    };

    let html = view::admin::theme_settings_page(&themes, &light_theme, &dark_theme);
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

pub async fn update_theme_settings(
    req: HttpRequest,
    server: web::Data<Server>,
    form: web::Form<ThemeSettingsForm>,
) -> Result<HttpResponse> {
    let user = get_user(req.clone());
    if let Some(user) = user.as_ref() {
        if !user.is_admin() {
            return Ok(HttpResponse::Forbidden()
                .content_type("text/html")
                .body("Access denied"));
        }
    } else {
        return Ok(HttpResponse::Forbidden()
            .content_type("text/html")
            .body("Access denied"));
    }

    // Validate the form
    if let Err(e) = form.validate() {
        log::error!("Theme settings validation failed: {:?}", e);
        return Ok(HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Invalid theme settings"));
    }

    // Update theme settings
    if let Err(e) = tokio::try_join!(
        db::theme::SystemThemeSettings::set_default_light_theme(
            &server.pg_pool,
            form.default_light_theme.clone()
        ),
        db::theme::SystemThemeSettings::set_default_dark_theme(
            &server.pg_pool,
            form.default_dark_theme.clone()
        )
    ) {
        log::error!("Failed to update theme settings: {}", e);
        return Err(ErrorInternalServerError("Failed to update theme settings"));
    }

    // Regenerate CSS cache
    if let Err(e) = db::theme::ThemeCssGenerator::generate_and_cache_css(&server.pg_pool).await {
        log::error!("Failed to regenerate CSS cache: {}", e);
        // Don't fail the request, just log the error
    }

    // Redirect back to theme settings page with success message
    Ok(HttpResponse::SeeOther()
        .append_header(("Location", "/admin/themes?updated=1"))
        .finish())
}
