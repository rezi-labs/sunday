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
pub async fn index_table_route(_server: web::Data<Server>, req: HttpRequest) -> AwResult<Markup> {
    let user = get_user(req.clone());
    Ok(placeholder_table(user.as_ref()))
}

#[get("/")]
pub async fn index_route(_server: web::Data<Server>, req: HttpRequest) -> AwResult<Markup> {
    let user = get_user(req.clone());

    // Return full page for normal requests
    Ok(index(placeholder_table(user.as_ref()), user.as_ref()))
}

#[get("/about/readme")]
pub async fn about_readme_endpoint(_server: web::Data<Server>) -> AwResult<Markup> {
    Ok(about::readme())
}

#[get("/about")]
pub async fn about_endpoint(_server: web::Data<Server>) -> AwResult<Markup> {
    Ok(about::about())
}

fn head() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            link rel="stylesheet" href="/assets/daisy.css";
            link rel="stylesheet" href="/assets/themes.css";
            link rel="stylesheet" href="/assets/tw.js";
            link rel="stylesheet" href="/assets/app.css";
            script src="/assets/htmx.js" {}
            script src="/assets/theme-switcher.js" {}
            title { "Sunday" }
        }
    }
}

pub fn index(content: Markup, user: Option<&crate::user::User>) -> Markup {
    html! {
       (head())
        body hx-boost="true" {
            (js("/assets/htmxListener.js"))
            (js("/assets/htmx-reload.js"))

            div class="min-h-screen bg-base-100" {
                (navbar::render(user))
                main class="container mx-auto px-4 py-6" {
                    (content)
                }
            }
        }
    }
}

fn placeholder_table(user: Option<&crate::user::User>) -> Markup {
    html! {
        div class="max-w-4xl mx-auto" id="main-page" {
            div class="card bg-base-200 shadow-xl" {
                div class="card-body" {
                    div class="flex justify-between items-center mb-6" {
                        h2 class="card-title" {
                            "Welcome to Sunday"
                        }
                    }

                    div class="bg-base-100 rounded-lg p-8 text-center" {
                        div class="mb-6" {
                            svg xmlns="http://www.w3.org/2000/svg" class="mx-auto h-16 w-16 text-base-content/50" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10";
                            }
                        }

                        h3 class="text-2xl font-semibold mb-4" { "Content Coming Soon" }

                        p class="text-base-content/70 mb-6" {
                            "This is a placeholder for the main content area. "
                            "Your application features will appear here."
                        }

                        @if let Some(user) = user {
                            p class="text-sm text-base-content/60" {
                                "Welcome back, " (user.email()) "!"
                            }
                        } @else {
                            div class="space-x-4" {
                                a href="/auth/login" class="btn btn-primary" {
                                    "Sign In"
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
