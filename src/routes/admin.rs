use crate::config::DatabaseClient;
use crate::routes::get_user;
use crate::{authentication, db, text_utils, view};
use actix_web::{HttpRequest, HttpResponse, Result, error::ErrorInternalServerError, web};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateUserForm {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub username: String,
    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,
}

#[derive(Serialize)]
pub struct AdminError {
    pub message: String,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub search: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct UpdateUserForm {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub username: String,
    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,
}

pub async fn admin_dashboard(
    req: HttpRequest,
    query: web::Query<SearchQuery>,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    let user = get_user(req);
    let db_client = db_client.as_ref();

    // Extract search term, filtering out empty strings
    let search_term = query
        .search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let users = db::user::list_all(db_client, search_term).await?;

    let html = view::admin::dashboard(users, user.as_ref(), search_term);
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

pub async fn create_user_form() -> Result<HttpResponse> {
    let html = view::admin::create_user_form();
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

pub async fn create_user(
    form: web::Form<CreateUserForm>,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    if let Err(errors) = form.validate() {
        let error_msg = errors
            .field_errors()
            .values()
            .flat_map(|v| v.iter())
            .map(|e| {
                e.message
                    .as_ref()
                    .unwrap_or(&std::borrow::Cow::Borrowed("Validation error"))
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(", ");

        return Ok(HttpResponse::BadRequest().json(AdminError { message: error_msg }));
    }

    let db_client = db_client.as_ref();

    // Check if user already exists
    let existing_user = db::user::find_by_email(&form.email, db_client).await?;
    if existing_user.is_some() {
        return Ok(HttpResponse::Conflict().json(AdminError {
            message: "User with this email already exists".to_string(),
        }));
    }

    let existing_username = db::user::find_by_username(&form.username, db_client).await?;
    if existing_username.is_some() {
        return Ok(HttpResponse::Conflict().json(AdminError {
            message: "Username already taken".to_string(),
        }));
    }

    // Generate a random password
    let generated_password = text_utils::generate_password(12);

    let password_hash = authentication::hash_password(&generated_password)
        .map_err(|_| ErrorInternalServerError("Password hashing failed"))?;

    db::user::create(
        form.username.clone(),
        form.email.clone(),
        db_client,
        password_hash,
    )
    .await?;

    let html = view::admin::user_created_success(&form.username, &form.email, &generated_password);
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

pub async fn delete_user(
    path: web::Path<String>,
    req: HttpRequest,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let current_user = get_user(req);
    let db_client = db_client.as_ref();

    // Parse UUID
    let user_uuid = match uuid::Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(AdminError {
                message: "Invalid user ID".to_string(),
            }));
        }
    };

    // Check if user exists and prevent deleting self
    if let Some(current) = &current_user
        && let Ok(current_uuid) = Uuid::parse_str(&current.id)
        && current_uuid == user_uuid
    {
        return Ok(HttpResponse::BadRequest().json(AdminError {
            message: "You cannot delete your own account".to_string(),
        }));
    }

    // Check if user exists
    let user_to_delete = db::user::find_by_id(&user_uuid, db_client).await?;
    if user_to_delete.is_none() {
        return Ok(HttpResponse::NotFound().json(AdminError {
            message: "User not found".to_string(),
        }));
    }

    // Delete the user
    db::user::delete_by_id(&user_uuid, db_client).await?;

    // Return the updated user table
    let users = db::user::list_all(db_client, None).await?;
    let html = view::admin::user_table_only(users);

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

pub async fn edit_user_form(
    path: web::Path<String>,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let db_client = db_client.as_ref();

    // Parse UUID
    let user_uuid = match uuid::Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(AdminError {
                message: "Invalid user ID".to_string(),
            }));
        }
    };

    // Find the user
    let user = db::user::find_by_id(&user_uuid, db_client).await?;
    let user = match user {
        Some(u) => u,
        None => {
            return Ok(HttpResponse::NotFound().json(AdminError {
                message: "User not found".to_string(),
            }));
        }
    };

    let html = view::admin::edit_user_form(&user);
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

pub async fn update_user(
    path: web::Path<String>,
    form: web::Form<UpdateUserForm>,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let db_client = db_client.as_ref();

    // Parse UUID
    let user_uuid = match uuid::Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(AdminError {
                message: "Invalid user ID".to_string(),
            }));
        }
    };

    // Validate form
    if let Err(errors) = form.validate() {
        let error_msg = errors
            .field_errors()
            .values()
            .flat_map(|v| v.iter())
            .map(|e| {
                e.message
                    .as_ref()
                    .unwrap_or(&std::borrow::Cow::Borrowed("Validation error"))
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(", ");

        return Ok(HttpResponse::BadRequest().json(AdminError { message: error_msg }));
    }

    // Check if user exists
    let user = db::user::find_by_id(&user_uuid, db_client).await?;
    if user.is_none() {
        return Ok(HttpResponse::NotFound().json(AdminError {
            message: "User not found".to_string(),
        }));
    }

    // Check if email is already taken by another user
    if let Some(existing_user) = db::user::find_by_email(&form.email, db_client).await?
        && existing_user.id != user_uuid
    {
        return Ok(HttpResponse::Conflict().json(AdminError {
            message: "Email is already taken by another user".to_string(),
        }));
    }

    // Check if username is already taken by another user
    if let Some(existing_user) = db::user::find_by_username(&form.username, db_client).await?
        && existing_user.id != user_uuid
    {
        return Ok(HttpResponse::Conflict().json(AdminError {
            message: "Username is already taken by another user".to_string(),
        }));
    }

    // Update the user
    db::user::update_user(&user_uuid, &form.username, &form.email, db_client).await?;

    // Return the updated user table
    let users = db::user::list_all(db_client, None).await?;
    let html = view::admin::user_table_only(users);

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

pub async fn toggle_user_active(
    path: web::Path<String>,
    req: HttpRequest,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let current_user = get_user(req);
    let db_client = db_client.as_ref();

    // Parse UUID
    let user_uuid = match uuid::Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(AdminError {
                message: "Invalid user ID".to_string(),
            }));
        }
    };

    // Prevent deactivating self
    if let Some(current) = &current_user
        && let Ok(current_uuid) = Uuid::parse_str(&current.id)
        && current_uuid == user_uuid
    {
        return Ok(HttpResponse::BadRequest().json(AdminError {
            message: "You cannot deactivate your own account".to_string(),
        }));
    }

    // Check if user exists
    let user = db::user::find_by_id(&user_uuid, db_client).await?;
    if user.is_none() {
        return Ok(HttpResponse::NotFound().json(AdminError {
            message: "User not found".to_string(),
        }));
    }

    // Toggle active status
    db::user::toggle_user_active_status(&user_uuid, db_client).await?;

    // Return the updated user table
    let users = db::user::list_all(db_client, None).await?;
    let html = view::admin::user_table_only(users);

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}

pub async fn toggle_user_admin(
    path: web::Path<String>,
    req: HttpRequest,
    db_client: web::Data<DatabaseClient>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let current_user = get_user(req);
    let db_client = db_client.as_ref();

    // Parse UUID
    let user_uuid = match uuid::Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(AdminError {
                message: "Invalid user ID".to_string(),
            }));
        }
    };

    // Prevent changing own admin status
    if let Some(current) = &current_user
        && let Ok(current_uuid) = Uuid::parse_str(&current.id)
        && current_uuid == user_uuid
    {
        return Ok(HttpResponse::BadRequest().json(AdminError {
            message: "You cannot change your own admin status".to_string(),
        }));
    }

    // Check if user exists
    let user = db::user::find_by_id(&user_uuid, db_client).await?;
    if user.is_none() {
        return Ok(HttpResponse::NotFound().json(AdminError {
            message: "User not found".to_string(),
        }));
    }

    // Toggle admin status
    db::user::toggle_user_admin_status(&user_uuid, db_client).await?;

    // Return the updated user table
    let users = db::user::list_all(db_client, None).await?;
    let html = view::admin::user_table_only(users);

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.into_string()))
}
