use crate::config::{DatabaseClient, Server};
use crate::db;
use crate::routes::get_user;
use crate::view;
use actix_web::{HttpRequest, HttpResponse, Result, error::ErrorInternalServerError, web};

pub async fn dashboard(
    server: web::Data<Server>,
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

    let html = view::admin::dashboard(users, user.as_ref(), None, &server);
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}
