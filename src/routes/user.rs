use actix_web::{HttpRequest, Result as AwResult, web};
use maud::Markup;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::Client;
use validator::Validate;

use crate::db;
use crate::routes::get_user;
use crate::view;

/// User dashboard handler
pub async fn dashboard(req: HttpRequest) -> AwResult<Markup> {
    let content = maud::html! {
        div class="text-center" {
            p { "Please log in to access your dashboard" }
            a href="/auth/login" class="btn btn-primary" { "Login" }
        };
    };
    let user = match get_user(req.clone()) {
        Some(user) => user,
        None => {
            return Ok(view::index(content, None));
        }
    };
    // Get user data from database
    let db_client = req
        .app_data::<web::Data<Arc<Mutex<Client>>>>()
        .expect("Database client not found");

    let db_user = match db::user::find_by_email(user.email(), db_client).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::error!("User not found in database: {}", user.email());
            return Ok(view::index(
                maud::html! {
                    div class="alert alert-error" {
                        "User not found in database"
                    }
                },
                Some(&user),
            ));
        }
        Err(e) => {
            log::error!("Error fetching user from database: {e:?}");
            return Ok(view::index(
                maud::html! {
                    div class="alert alert-error" {
                        "Error loading user data"
                    }
                },
                Some(&user),
            ));
        }
    };

    Ok(view::index(
        view::user::dashboard(&db_user, &user),
        Some(&user),
    ))
}

/// User profile edit form handler
pub async fn edit_profile_form(req: HttpRequest) -> AwResult<Markup> {
    let user = match get_user(req.clone()) {
        Some(user) => user,
        None => {
            return Ok(view::index(
                maud::html! {
                    div class="text-center" {
                        p { "Please log in to edit your profile" }
                        a href="/auth/login" class="btn btn-primary" { "Login" }
                    }
                },
                None,
            ));
        }
    };

    // Get user data from database
    let db_client = req
        .app_data::<web::Data<Arc<Mutex<Client>>>>()
        .expect("Database client not found");

    let db_user = match db::user::find_by_email(user.email(), db_client).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::error!("User not found in database: {}", user.email());
            return Ok(view::index(
                maud::html! {
                    div class="alert alert-error" {
                        "User not found in database"
                    }
                },
                Some(&user),
            ));
        }
        Err(e) => {
            log::error!("Error fetching user from database: {e:?}");
            return Ok(view::index(
                maud::html! {
                    div class="alert alert-error" {
                        "Error loading user data"
                    }
                },
                Some(&user),
            ));
        }
    };

    Ok(view::index(
        view::user::edit_profile_form(&db_user),
        Some(&user),
    ))
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProfileForm {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// Update user profile handler
pub async fn update_profile(
    req: HttpRequest,
    form: web::Json<UpdateProfileForm>,
) -> AwResult<Markup> {
    let user = match get_user(req.clone()) {
        Some(user) => user,
        None => {
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Unauthorized"
                }
            });
        }
    };

    // Validate form
    if let Err(errors) = form.validate() {
        log::warn!("Profile update validation failed: {errors:?}");
        return Ok(maud::html! {
            div class="alert alert-error" {
                ul {
                    @for (_field, field_errors) in errors.field_errors() {
                        @for error in field_errors {
                            li { (error.message.as_ref().unwrap_or(&"Validation error".into())) }
                        }
                    }
                }
            }
        });
    }

    let db_client = req
        .app_data::<web::Data<Arc<Mutex<Client>>>>()
        .expect("Database client not found");

    // Get current user from database
    let db_user = match db::user::find_by_email(user.email(), db_client).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::error!("User not found in database: {}", user.email());
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "User not found"
                }
            });
        }
        Err(e) => {
            log::error!("Error fetching user from database: {e:?}");
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Error loading user data"
                }
            });
        }
    };

    // Update user profile
    let updated_user = db::user::Model {
        id: db_user.id,
        username: form.username.clone(),
        email: form.email.clone(),
        created_at: db_user.created_at,
        updated_at: Some(chrono::Utc::now()),
        last_login_at: db_user.last_login_at,
        is_active: db_user.is_active,
        is_admin: db_user.is_admin,
    };

    match db::user::update(&updated_user, db_client).await {
        Ok(_) => {
            log::info!("User profile updated successfully: {}", updated_user.email);
            Ok(view::user::dashboard_with_message(
                &updated_user,
                &user,
                Some("Profile updated successfully!"),
            ))
        }
        Err(e) => {
            log::error!("Error updating user profile: {e:?}");
            Ok(maud::html! {
                div class="alert alert-error" {
                    "Error updating profile"
                }
            })
        }
    }
}
