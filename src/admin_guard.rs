use crate::user::User;
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{Ready, ready},
    rc::Rc,
};

pub struct AdminGuard;

impl<S, B> Transform<S, ServiceRequest> for AdminGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AdminGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminGuardMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AdminGuardMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Check if user is authenticated and is admin
            let user = req.extensions().get::<User>().cloned();

            match user {
                Some(user) => {
                    if user.is_admin() {
                        // User is admin, proceed with request
                        let res = service.call(req).await?;
                        Ok(res.map_into_left_body())
                    } else {
                        // User is not admin, return 403
                        log::warn!(
                            "Non-admin user {} attempted to access admin route",
                            user.email()
                        );
                        let (req, _) = req.into_parts();
                        let response = HttpResponse::Forbidden()
                            .content_type("text/html")
                            .body(r#"
                                <!DOCTYPE html>
                                <html>
                                <head>
                                    <title>Access Denied</title>
                                    <meta charset="UTF-8">
                                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                                </head>
                                <body>
                                    <h1>Access Denied</h1>
                                    <p>You do not have permission to access this resource.</p>
                                    <a href="/">Return to Home</a>
                                </body>
                                </html>
                            "#);
                        Ok(ServiceResponse::new(req, response).map_into_right_body())
                    }
                }
                None => {
                    // User is not authenticated, redirect to login
                    let (req, _) = req.into_parts();
                    let response = HttpResponse::Found()
                        .append_header(("Location", "/auth/login"))
                        .finish();
                    Ok(ServiceResponse::new(req, response).map_into_right_body())
                }
            }
        })
    }
}
