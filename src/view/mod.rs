use actix_web::{HttpRequest, Result as AwResult, get, web};
use maud::{Markup, html};

pub mod about;
pub mod admin;
pub mod auth;
mod icons;
mod navbar;
pub mod user;

use crate::config::Server;
use crate::routes::get_user;

#[get("/table")]
pub async fn index_table_route(server: web::Data<Server>, req: HttpRequest) -> AwResult<Markup> {
    let user = get_user(req.clone());
    Ok(placeholder_table(user.as_ref(), &server))
}

#[get("/")]
pub async fn index_route(server: web::Data<Server>, req: HttpRequest) -> AwResult<Markup> {
    let user = get_user(req.clone());

    // Return full page for normal requests
    Ok(index(
        placeholder_table(user.as_ref(), &server),
        user.as_ref(),
        &server,
    ))
}

#[get("/about/readme")]
pub async fn about_readme_endpoint(_server: web::Data<Server>) -> AwResult<Markup> {
    Ok(about::readme())
}

#[get("/about")]
pub async fn about_endpoint(_server: web::Data<Server>) -> AwResult<Markup> {
    Ok(about::about())
}

fn head(server: &Server) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            link rel="stylesheet" href="/assets/daisy.css";
            link rel="stylesheet" href="/api/themes/css";
            link rel="stylesheet" href="/assets/app.css";
            (js("/assets/htmx.js"))
            (js("/assets/tw.js"))
            (js_module("/assets/deepchat.js"))
            title { (server.sunday_name()) }
        }
    }
}

pub fn index(content: Markup, user: Option<&crate::user::User>, server: &Server) -> Markup {
    html! {
       (head(server))
        body hx-boost="true" {


            div class="min-h-screen bg-base-100" {
                (navbar::render(user, server))
                main class="container mx-auto px-4 py-6" {
                    (content)
                }
            }
        }
    }
}

fn placeholder_table(user: Option<&crate::user::User>, server: &Server) -> Markup {
    html! {
        div class="max-w-4xl mx-auto" id="main-page" {
            div class="card bg-base-200 shadow-xl" {
                div class="card-body" {
                    div class="flex justify-between items-center mb-6" {
                        h2 class="card-title" {
                            "Welcome to " (server.sunday_name())
                        }
                    }

                    // Chat Interface
                    div class="bg-base-100 rounded-lg p-4" {
                        @if let Some(user) = user {

                            // Deep Chat Component
                            deep-chat
                                style="width: 400px; height: 400px; border-radius: 8px;"
                                connect=r#"{"url": "/api/chat", "method": "POST"}"#
                                intro-message=(format!(r#"{{"text": "Hello! I'm {} AI. How can I help you today?"}}"#, server.sunday_name()))
                            {}
                        } @else {
                            div class="text-center" {
                                div class="mb-6" {
                                    svg xmlns="http://www.w3.org/2000/svg" class="mx-auto h-16 w-16 text-base-content/50" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.418 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.418-8 9-8s9 3.582 9 8z";
                                    }
                                }
                                h3 class="text-2xl font-semibold mb-4" { "Chat with " (server.sunday_name()) " AI" }
                                p class="text-base-content/70 mb-6" {
                                    "Please sign in to start chatting with " (server.sunday_name()) " AI."
                                }
                                div class="space-x-4" {
                                    a href="/auth/login" class="btn btn-primary" {
                                        "Sign In to Chat"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn js(src: &str) -> Markup {
    html! {
        script src=(src) {}
    }
}

fn js_module(src: &str) -> Markup {
    html! {
        script type="module" src=(src) {}
    }
}
