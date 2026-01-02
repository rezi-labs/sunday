use actix_web::{HttpRequest, Result as AwResult, web};
use maud::Markup;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::Client;
use uuid::Uuid;
use validator::Validate;

use crate::db;
use crate::routes::get_user;
use crate::view;

/// Create user form handler
pub async fn create_user_form(req: HttpRequest) -> AwResult<Markup> {
    let _user = match get_user(req) {
        Some(user) if user.is_admin() => user,
        _ => {
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Unauthorized - Admin access required"
                }
            });
        }
    };

    Ok(maud::html! {
        div class="max-w-2xl mx-auto p-6" {
            div class="flex items-center gap-4 mb-6" {
                button hx-get="/admin"
                       hx-target="body"
                       class="btn btn-ghost btn-sm" {
                    svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18";
                    }
                    "Back to Users"
                }
                h2 class="text-2xl font-bold" { "Create New User" }
            }

            div class="card bg-base-100 shadow-xl" {
                div class="card-body" {
                    form hx-post="/admin/users"
                          hx-target="#main-content"
                          hx-swap="innerHTML"
                          class="space-y-4" {

                        div class="form-control" {
                            label class="label" {
                                span class="label-text font-semibold" { "Username *" }
                            }
                            input type="text"
                                  name="username"
                                  placeholder="Enter username"
                                  class="input input-bordered w-full"
                                  required;
                        }

                        div class="form-control" {
                            label class="label" {
                                span class="label-text font-semibold" { "Email *" }
                            }
                            input type="email"
                                  name="email"
                                  placeholder="Enter email address"
                                  class="input input-bordered w-full"
                                  required;
                        }

                        div class="form-control" {
                            label class="label cursor-pointer justify-start gap-4" {
                                input type="checkbox" name="is_admin" class="checkbox checkbox-primary";
                                span class="label-text" { "Administrator privileges" }
                            }
                        }

                        div class="card-actions justify-end pt-4" {
                            button type="button"
                                   hx-get="/admin"
                                   hx-target="body"
                                   class="btn btn-ghost" {
                                "Cancel"
                            }
                            button type="submit" class="btn btn-primary" {
                                "Create User"
                            }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserForm {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub is_admin: Option<String>,
}

/// Create user handler
pub async fn create_user(req: HttpRequest, form: web::Form<CreateUserForm>) -> AwResult<Markup> {
    let user = match get_user(req.clone()) {
        Some(user) if user.is_admin() => user,
        _ => {
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Unauthorized - Admin access required"
                }
            });
        }
    };

    // Validate form
    if let Err(errors) = form.validate() {
        log::warn!("Create user validation failed: {errors:?}");
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

    // Check if username or email already exists
    if let Ok(Some(_)) = db::user::find_by_username(&form.username, db_client).await {
        return Ok(maud::html! {
            div class="alert alert-error" {
                "Username already exists"
            }
        });
    }

    if let Ok(Some(_)) = db::user::find_by_email(&form.email, db_client).await {
        return Ok(maud::html! {
            div class="alert alert-error" {
                "Email already exists"
            }
        });
    }

    let is_admin = form.is_admin.as_deref() == Some("on");

    // Create the user
    let result = if is_admin {
        db::user::create_admin(form.username.clone(), form.email.clone(), db_client).await
    } else {
        db::user::create(
            form.username.clone(),
            form.email.clone(),
            db_client,
            true, // is_active
        )
        .await
    };

    match result {
        Ok(_) => {
            log::info!(
                "User created successfully by {}: {} ({})",
                user.email(),
                form.username,
                form.email
            );

            // Redirect back to admin dashboard with success message
            let users = db::user::list_all(db_client, None)
                .await
                .unwrap_or_default();
            Ok(view::admin::dashboard::dashboard(users, Some(&user), None))
        }
        Err(e) => {
            log::error!("Error creating user: {e:?}");
            Ok(maud::html! {
                div class="alert alert-error" {
                    "Error creating user"
                }
            })
        }
    }
}

/// Edit user form handler
pub async fn edit_user_form(req: HttpRequest, path: web::Path<Uuid>) -> AwResult<Markup> {
    let _user = match get_user(req.clone()) {
        Some(user) if user.is_admin() => user,
        _ => {
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Unauthorized - Admin access required"
                }
            });
        }
    };

    let user_id = path.into_inner();
    let db_client = req
        .app_data::<web::Data<Arc<Mutex<Client>>>>()
        .expect("Database client not found");

    let db_user = match db::user::find_by_id(&user_id, db_client).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "User not found"
                }
            });
        }
        Err(e) => {
            log::error!("Error fetching user: {e:?}");
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Error loading user"
                }
            });
        }
    };

    Ok(maud::html! {
        div class="max-w-2xl mx-auto p-6" {
            div class="flex items-center gap-4 mb-6" {
                button hx-get="/admin"
                       hx-target="body"
                       class="btn btn-ghost btn-sm" {
                    svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18";
                    }
                    "Back to Users"
                }
                h2 class="text-2xl font-bold" { "Edit User: " (db_user.username) }
            }

            div class="card bg-base-100 shadow-xl" {
                div class="card-body" {
                    form hx-put=(format!("/admin/users/{}", db_user.id))
                          hx-target="#main-content"
                          hx-swap="innerHTML"
                          class="space-y-4" {

                        div class="form-control" {
                            label class="label" {
                                span class="label-text font-semibold" { "Username *" }
                            }
                            input type="text"
                                  name="username"
                                  value=(db_user.username)
                                  class="input input-bordered w-full"
                                  required;
                        }

                        div class="form-control" {
                            label class="label" {
                                span class="label-text font-semibold" { "Email *" }
                            }
                            input type="email"
                                  name="email"
                                  value=(db_user.email)
                                  class="input input-bordered w-full"
                                  required;
                        }

                        div class="form-control" {
                            label class="label cursor-pointer justify-start gap-4" {
                                input type="checkbox" name="is_admin" class="checkbox checkbox-primary"
                                      checked[db_user.is_admin];
                                span class="label-text" { "Administrator privileges" }
                            }
                        }

                        div class="form-control" {
                            label class="label cursor-pointer justify-start gap-4" {
                                input type="checkbox" name="is_active" class="checkbox checkbox-primary"
                                      checked[db_user.is_active];
                                span class="label-text" { "Active account" }
                            }
                        }

                        div class="card-actions justify-end pt-4" {
                            button type="button"
                                   hx-get="/admin"
                                   hx-target="body"
                                   class="btn btn-ghost" {
                                "Cancel"
                            }
                            button type="submit" class="btn btn-primary" {
                                "Update User"
                            }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserForm {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub is_admin: Option<String>,
    pub is_active: Option<String>,
}

/// Update user handler
pub async fn update_user(
    req: HttpRequest,
    path: web::Path<Uuid>,
    form: web::Form<UpdateUserForm>,
) -> AwResult<Markup> {
    let user = match get_user(req.clone()) {
        Some(user) if user.is_admin() => user,
        _ => {
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Unauthorized - Admin access required"
                }
            });
        }
    };

    // Validate form
    if let Err(errors) = form.validate() {
        log::warn!("Update user validation failed: {errors:?}");
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

    let user_id = path.into_inner();
    let db_client = req
        .app_data::<web::Data<Arc<Mutex<Client>>>>()
        .expect("Database client not found");

    // Update basic user info first
    match db::user::update_user(&user_id, &form.username, &form.email, db_client).await {
        Ok(_) => {
            // Handle admin/active status toggles separately
            let current_user = match db::user::find_by_id(&user_id, db_client).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Ok(maud::html! {
                        div class="alert alert-error" {
                            "User not found after update"
                        }
                    });
                }
                Err(e) => {
                    log::error!("Failed to fetch updated user: {e:?}");
                    return Ok(maud::html! {
                        div class="alert alert-error" {
                            "Error fetching updated user"
                        }
                    });
                }
            };

            let should_be_admin = form.is_admin.as_deref() == Some("on");
            let should_be_active = form.is_active.as_deref() == Some("on");

            // Toggle admin status if needed
            if current_user.is_admin != should_be_admin {
                if let Err(e) = db::user::toggle_user_admin_status(&user_id, db_client).await {
                    log::error!("Failed to toggle admin status: {e:?}");
                }
            }

            // Toggle active status if needed
            if current_user.is_active != should_be_active {
                if let Err(e) = db::user::toggle_user_active_status(&user_id, db_client).await {
                    log::error!("Failed to toggle active status: {e:?}");
                }
            }

            log::info!(
                "User updated successfully by {}: {} ({})",
                user.email(),
                form.username,
                form.email
            );

            // Redirect back to admin dashboard
            let users = db::user::list_all(db_client, None)
                .await
                .unwrap_or_default();
            Ok(view::admin::dashboard::dashboard(users, Some(&user), None))
        }
        Err(e) => {
            log::error!("Error updating user: {e:?}");
            Ok(maud::html! {
                div class="alert alert-error" {
                    "Error updating user"
                }
            })
        }
    }
}

/// Delete user handler
pub async fn delete_user(req: HttpRequest, path: web::Path<Uuid>) -> AwResult<Markup> {
    let user = match get_user(req.clone()) {
        Some(user) if user.is_admin() => user,
        _ => {
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Unauthorized - Admin access required"
                }
            });
        }
    };

    let user_id = path.into_inner();
    let db_client = req
        .app_data::<web::Data<Arc<Mutex<Client>>>>()
        .expect("Database client not found");

    match db::user::delete_by_id(&user_id, db_client).await {
        Ok(_) => {
            log::info!("User deleted successfully by {}: {}", user.email(), user_id);

            // Return updated user list
            let users = db::user::list_all(db_client, None)
                .await
                .unwrap_or_default();
            Ok(view::admin::dashboard::dashboard(users, Some(&user), None))
        }
        Err(e) => {
            log::error!("Error deleting user: {e:?}");
            Ok(maud::html! {
                div class="alert alert-error" {
                    "Error deleting user"
                }
            })
        }
    }
}

/// Toggle user active status
pub async fn toggle_user_active(req: HttpRequest, path: web::Path<Uuid>) -> AwResult<Markup> {
    let user = match get_user(req.clone()) {
        Some(user) if user.is_admin() => user,
        _ => {
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Unauthorized - Admin access required"
                }
            });
        }
    };

    let user_id = path.into_inner();
    let db_client = req
        .app_data::<web::Data<Arc<Mutex<Client>>>>()
        .expect("Database client not found");

    match db::user::toggle_user_active_status(&user_id, db_client).await {
        Ok(_) => {
            log::info!(
                "User active status toggled by {}: {}",
                user.email(),
                user_id
            );

            // Return updated user list
            let users = db::user::list_all(db_client, None)
                .await
                .unwrap_or_default();
            Ok(view::admin::dashboard::dashboard(users, Some(&user), None))
        }
        Err(e) => {
            log::error!("Error toggling user active status: {e:?}");
            Ok(maud::html! {
                div class="alert alert-error" {
                    "Error updating user status"
                }
            })
        }
    }
}

/// Toggle user admin status
pub async fn toggle_user_admin(req: HttpRequest, path: web::Path<Uuid>) -> AwResult<Markup> {
    let user = match get_user(req.clone()) {
        Some(user) if user.is_admin() => user,
        _ => {
            return Ok(maud::html! {
                div class="alert alert-error" {
                    "Unauthorized - Admin access required"
                }
            });
        }
    };

    let user_id = path.into_inner();
    let db_client = req
        .app_data::<web::Data<Arc<Mutex<Client>>>>()
        .expect("Database client not found");

    match db::user::toggle_user_admin_status(&user_id, db_client).await {
        Ok(_) => {
            log::info!("User admin status toggled by {}: {}", user.email(), user_id);

            // Return updated user list
            let users = db::user::list_all(db_client, None)
                .await
                .unwrap_or_default();
            Ok(view::admin::dashboard::dashboard(users, Some(&user), None))
        }
        Err(e) => {
            log::error!("Error toggling user admin status: {e:?}");
            Ok(maud::html! {
                div class="alert alert-error" {
                    "Error updating user admin status"
                }
            })
        }
    }
}
