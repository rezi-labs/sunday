use crate::user::User;
use actix_web::{HttpMessage, HttpRequest};

pub mod admin;
pub mod assets;
pub mod auth;
pub mod technical;
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
