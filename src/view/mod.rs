use actix_web::{HttpRequest, HttpResponse, Result as AwResult, get, web};
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
    Ok(placeholder_table(user.as_ref(), None, &server))
}

#[get("/")]
pub async fn index_route(server: web::Data<Server>, req: HttpRequest) -> AwResult<HttpResponse> {
    let user = get_user(req.clone());
    let is_global_route = req.path() == "/";

    // For index route, redirect authenticated users to chat
    if is_global_route {
        if let Some(user) = user.as_ref() {
            // Redirect to tenant chat if user has a tenant, otherwise to acme in local mode
            let redirect_path = if let Some(tenant) = user.tenant() {
                format!("/{}/chat", tenant)
            } else if server.local() {
                "/acme/chat".to_string()
            } else {
                // No tenant context, show login
                "/auth/login".to_string()
            };

            return Ok(HttpResponse::SeeOther()
                .append_header(("Location", redirect_path))
                .finish());
        }
    }

    // Return full page for normal requests
    let tenant_context = None;
    let index = index(
        placeholder_table(user.as_ref(), tenant_context, &server),
        tenant_context,
        user.as_ref(),
        &server,
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(index.into_string()))
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

pub fn index(
    content: Markup,
    tenant: Option<&str>,
    user: Option<&crate::user::User>,
    server: &Server,
) -> Markup {
    html! {
       (head(server))
        body hx-boost="true" {

            div class="min-h-screen bg-base-100" {
                (navbar::render(user, server, tenant))
                main class="container mx-auto px-4 py-6" {
                    (content)
                }
            }
        }
    }
}

fn placeholder_table(
    user: Option<&crate::user::User>,
    tenant: Option<&str>,
    server: &Server,
) -> Markup {
    let chat_url = if let Some(t) = tenant {
        format!("/{}/api/chat", t)
    } else if server.local() {
        "/acme/api/chat".to_string()
    } else {
        "#".to_string()
    };

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
                        @if let Some(_user) = user {
                            // Deep Chat Component
                            deep-chat
                                style="width: 400px; height: 400px; border-radius: 8px;"
                                connect=(format!(r#"{{"url": "{}", "method": "POST"}}"#, chat_url))
                                intro-message=(format!(r#"{{"text": "Hello! I'm {} AI. How can I help you today?"}}"#, server.sunday_name()))
                            {}
                        } @else {
                            div class="text-center" {
                                p { "Please log in to access the chat interface" }
                                @if server.local() && tenant.is_none() {
                                    p class="text-sm text-base-content/60 mt-4" {
                                        "In local mode, the chat will be available at "
                                        a href="/acme/dashboard" class="link" { "/acme/" }
                                        "after login."
                                    }
                                }
                                @if chat_url != "#" {
                                    a href=(if let Some(t) = tenant { format!("/{}/auth/login", t) } else { "/auth/login".to_string() }) class="btn btn-primary" { "Login" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
