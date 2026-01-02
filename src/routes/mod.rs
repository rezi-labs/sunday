use crate::config::DatabaseClient;
use crate::tenant_middleware::{get_tenant_db, get_tenant_name};
use crate::user::User;
use actix_web::{HttpMessage, HttpRequest, web};

pub mod admin;
pub mod assets;
pub mod auth;
pub mod chat;
pub mod technical;
pub mod tenants;
pub mod user;

pub fn get_user(req: HttpRequest) -> Option<User> {
    let user = req.extensions().get::<User>().cloned();
    match &user {
        Some(u) => log::debug!("User found in request extensions: {}", u.email()),
        None => log::debug!(
            "No user found in request extensions for path: {}",
            req.path()
        ),
    }
    user
}

/// Get database client for the current tenant from request extensions
/// Falls back to global database client if no tenant-specific client is available
pub fn get_database_client(req: &HttpRequest) -> Option<DatabaseClient> {
    // Try to get tenant-specific database client first
    if let Some(tenant_db) = get_tenant_db(req) {
        log::debug!(
            "Using tenant-specific database client for tenant: {:?}",
            get_tenant_name(req)
        );
        return Some(tenant_db);
    }

    // Fall back to global database client
    log::debug!("Using global database client");
    req.app_data::<web::Data<DatabaseClient>>()
        .map(|data| data.get_ref().clone())
}
