use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct User {
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub user_name: String,
    pub email: String,
    pub is_admin: bool,
    pub tenant: Option<String>,
}

impl User {
    pub fn new(id: String, email: String, user_name: String, is_admin: bool) -> Self {
        User {
            id,
            email,
            user_name,
            is_admin,
            tenant: None,
        }
    }

    pub fn with_tenant(
        id: String,
        email: String,
        user_name: String,
        is_admin: bool,
        tenant: Option<String>,
    ) -> Self {
        User {
            id,
            email,
            user_name,
            is_admin,
            tenant,
        }
    }

    pub fn _id(&self) -> &str {
        &self.id
    }

    #[allow(dead_code)]
    pub fn user_name(&self) -> &str {
        &self.user_name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    pub fn tenant(&self) -> Option<&str> {
        self.tenant.as_deref()
    }

    pub fn _initials(&self) -> String {
        let mut split = self.email().split(".");
        let first_name = split.next().unwrap_or("Unknown");
        let last_name = split.next().unwrap_or("User");
        format!(
            "{}{}",
            first_name.chars().next().unwrap_or(' ').to_uppercase(),
            last_name.chars().next().unwrap_or(' ').to_uppercase()
        )
    }
}

use actix_session::SessionExt;
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    web,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{Ready, ready},
    rc::Rc,
};

pub struct UserExtractor;

impl<S, B> Transform<S, ServiceRequest> for UserExtractor
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UserExtractorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UserExtractorMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct UserExtractorMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for UserExtractorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Extract user from session if authenticated
            let session = req.get_session();
            log::debug!(
                "Checking for session authentication on path: {}",
                req.path()
            );

            // Try to get all session entries for debugging
            log::debug!(
                "Session entries for path {}: {:?}",
                req.path(),
                session.entries()
            );

            // Extract tenant from URL path (first segment after /)
            let path = req.path();
            let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
            let route_tenant = if !path_segments.is_empty()
                && !path.starts_with("/health")
                && !path.starts_with("/api/tenants")
                && !path.starts_with("/assets")
                && !path.starts_with("/auth")
                && path != "/"
            {
                // Validate tenant name format
                let tenant_candidate = path_segments[0];
                if tenant_candidate
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_')
                {
                    // Check if user has access to this tenant
                    match session.get::<Vec<String>>("allowed_tenants") {
                        Ok(Some(allowed_tenants)) => {
                            if allowed_tenants.contains(&tenant_candidate.to_string()) {
                                log::debug!(
                                    "User has access to tenant from route: {}",
                                    tenant_candidate
                                );
                                Some(tenant_candidate.to_string())
                            } else {
                                log::warn!(
                                    "User does not have access to tenant: {}",
                                    tenant_candidate
                                );
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Fallback to default tenant from session if no route tenant
            let tenant =
                route_tenant.or_else(|| session.get::<String>("default_tenant").ok().flatten());

            match session.get::<String>("sunday_session_id") {
                Ok(Some(session_id)) => {
                    log::debug!("Found session ID in cookie: {session_id}");

                    // Get database client from app data
                    if let Some(db_client) =
                        req.app_data::<web::Data<crate::config::DatabaseClient>>()
                    {
                        match crate::db::session::validate(&session_id, db_client.as_ref()).await {
                            Ok(Some(user_model)) => {
                                log::info!(
                                    "User authenticated: {} ({}) with tenant: {:?}",
                                    user_model.email,
                                    user_model.id,
                                    tenant
                                );
                                let user = User::with_tenant(
                                    user_model.id.to_string(),
                                    user_model.email,
                                    user_model.username.to_string(),
                                    user_model.is_admin,
                                    tenant.clone(),
                                );
                                req.extensions_mut().insert(user);
                            }
                            Ok(None) => {
                                log::info!(
                                    "Session validation failed for ID: {session_id} - clearing session"
                                );
                                if let Some(e) = session.remove("sunday_session_id") {
                                    log::warn!("Failed to remove invalid session: {e:?}");
                                }
                            }
                            Err(e) => {
                                log::error!(
                                    "Database error during session validation: {e:?} - clearing session"
                                );
                                if let Some(e) = session.remove("sunday_session_id") {
                                    log::warn!("Failed to remove session after DB error: {e:?}");
                                }
                            }
                        }
                    } else {
                        log::error!("Database client not available in app data");
                    }
                }
                Ok(None) => {
                    // No regular session, check for OIDC session
                    if let Some(oidc_user) = crate::oidc::get_user_from_session(&session) {
                        log::debug!(
                            "Found OIDC user in session: {} with tenant: {:?}",
                            oidc_user.email,
                            tenant
                        );
                        let user = User::with_tenant(
                            oidc_user.sub,
                            oidc_user.email,
                            oidc_user.name.unwrap_or_else(|| "OIDC User".to_string()),
                            false, // OIDC users are not admin by default - can be updated later
                            tenant.clone(),
                        );
                        req.extensions_mut().insert(user);
                    } else {
                        log::debug!(
                            "No 'sunday_session_id' found in cookie for path: {}",
                            req.path()
                        );
                    }
                }
                Err(e) => {
                    log::warn!("Error retrieving 'sunday_session_id' from cookie: {e:?}");
                }
            }

            service.call(req).await
        })
    }
}
