use crate::config::Server;
use crate::tenant::{TenantDatabaseClient, TenantManager};
use crate::user::User;
use actix_session::SessionExt;
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{Ready, ready},
    rc::Rc,
    sync::Arc,
};

pub struct TenantMiddleware {
    tenant_manager: Arc<TenantManager>,
    config: Server,
}

impl TenantMiddleware {
    pub fn new(tenant_manager: Arc<TenantManager>, config: Server) -> Self {
        Self {
            tenant_manager,
            config,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TenantMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TenantMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TenantMiddlewareService {
            service: Rc::new(service),
            tenant_manager: self.tenant_manager.clone(),
            config: self.config.clone(),
        }))
    }
}

pub struct TenantMiddlewareService<S> {
    service: Rc<S>,
    tenant_manager: Arc<TenantManager>,
    config: Server,
}

impl<S, B> Service<ServiceRequest> for TenantMiddlewareService<S>
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
        let tenant_manager = self.tenant_manager.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // Skip tenant validation for global routes (health, tenant management, assets, auth)
            let path = req.path();
            if path.starts_with("/health")
                || path.starts_with("/api/tenants")
                || path.starts_with("/assets")
                || path.starts_with("/auth")
                || path == "/"
            {
                return service.call(req).await;
            }

            // Extract tenant from URL path
            let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

            if path_segments.is_empty() {
                log::warn!("Tenant not specified in URL for path: {}", path);
                // Continue with request - let the route handler deal with missing tenant
                return service.call(req).await;
            }

            let tenant_name = path_segments[0];

            // Validate tenant name format
            if !tenant_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                log::warn!("Invalid tenant name format: {}", tenant_name);
                // Continue with request - let the route handler deal with invalid tenant
                return service.call(req).await;
            }

            // Check if user has access to this tenant
            let session = req.get_session();

            // In local mode, allow access to any tenant for authenticated users
            if config.local() {
                if req.extensions().get::<User>().is_some() {
                    // User is authenticated, allow access in local mode
                } else {
                    log::warn!("Authentication required for tenant access in local mode");
                    // Continue with request - let the route handler deal with auth
                    return service.call(req).await;
                }
            } else {
                // In production mode, check session for tenant access
                if !validate_tenant_access(&session, tenant_name, &config).await {
                    log::warn!("Access denied to tenant '{}' for user", tenant_name);
                    // Continue with request - let the route handler deal with access denied
                    return service.call(req).await;
                }
            }

            // Get or create tenant database connection
            match tenant_manager.get_tenant_connection(tenant_name).await {
                Ok(tenant_db) => {
                    // Store tenant info and database connection in request extensions
                    req.extensions_mut().insert(tenant_name.to_string());
                    req.extensions_mut().insert(tenant_db);

                    // Continue with the request
                    return service.call(req).await;
                }
                Err(e) => {
                    log::error!(
                        "Failed to get tenant connection for '{}': {:?}",
                        tenant_name,
                        e
                    );
                    // Continue with request - let the route handler deal with missing tenant
                    return service.call(req).await;
                }
            }
        })
    }
}

async fn validate_tenant_access(
    session: &actix_session::Session,
    tenant_name: &str,
    _config: &Server,
) -> bool {
    // Try to get allowed tenants from session
    match session.get::<Vec<String>>("allowed_tenants") {
        Ok(Some(allowed_tenants)) => {
            log::debug!("User allowed tenants: {:?}", allowed_tenants);
            allowed_tenants.contains(&tenant_name.to_string())
        }
        Ok(None) => {
            log::warn!(
                "No tenant permissions found in session for tenant '{}'",
                tenant_name
            );
            false
        }
        Err(e) => {
            log::error!("Failed to read tenant permissions from session: {:?}", e);
            false
        }
    }
}

// Helper function to extract tenant database client from request
pub fn get_tenant_db(req: &actix_web::HttpRequest) -> Option<TenantDatabaseClient> {
    req.extensions().get::<TenantDatabaseClient>().cloned()
}

// Helper function to extract tenant name from request
pub fn get_tenant_name(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions().get::<String>().cloned()
}
