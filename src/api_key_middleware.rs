use actix_web::{
    FromRequest, HttpMessage, HttpRequest, Result,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{Ready, ready},
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AuthType {
    ApiKey,
}

#[derive(Debug, Clone)]
pub enum AuthState {
    Authenticated(AuthType),
    Unauthenticated,
}

impl AuthState {
    /// Check if the request is authenticated
    #[allow(dead_code)]
    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthState::Authenticated(_))
    }
}

/// Implement FromRequest so AuthState can be used as an extractor in handlers
impl FromRequest for AuthState {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_state = req
            .extensions()
            .get::<AuthState>()
            .cloned()
            .unwrap_or(AuthState::Unauthenticated);

        // Fail with 403 Forbidden if unauthenticated
        match auth_state {
            AuthState::Authenticated(_) => ready(Ok(auth_state)),
            AuthState::Unauthenticated => {
                log::warn!(
                    "Unauthenticated access attempt to protected endpoint: {}",
                    req.path()
                );
                ready(Err(actix_web::error::ErrorForbidden(
                    "Authentication required. Please provide a valid API key.",
                )))
            }
        }
    }
}

/// Helper function to get auth state from request
#[allow(dead_code)]
pub fn get_auth_state(req: &HttpRequest) -> AuthState {
    req.extensions()
        .get::<AuthState>()
        .cloned()
        .unwrap_or(AuthState::Unauthenticated)
}

pub struct ApiKeyAuth {
    pub api_keys: Vec<String>,
}

impl ApiKeyAuth {
    pub fn new(api_keys: Vec<String>) -> Self {
        Self { api_keys }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = ApiKeyAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyAuthMiddleware {
            service: Rc::new(service),
            api_keys: self.api_keys.clone(),
        }))
    }
}

pub struct ApiKeyAuthMiddleware<S> {
    service: Rc<S>,
    api_keys: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let api_keys = self.api_keys.clone();

        Box::pin(async move {
            // Extract API key from Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            let auth_state = match auth_header {
                Some(header) if header.starts_with("Bearer ") => {
                    let provided_key = &header[7..];

                    // Validate API key against all valid keys
                    if api_keys.iter().any(|key| key == provided_key) {
                        log::debug!("API key validated successfully for path: {}", req.path());
                        AuthState::Authenticated(AuthType::ApiKey)
                    } else {
                        log::warn!("Invalid API key provided for path: {}", req.path());
                        AuthState::Unauthenticated
                    }
                }
                _ => {
                    log::debug!("No valid Authorization header for path: {}", req.path());
                    AuthState::Unauthenticated
                }
            };

            // Inject auth state into request extensions
            req.extensions_mut().insert(auth_state);

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}
