use crate::config::Server;

pub fn login_form(message: Option<&str>, server: &Server) -> maud::Markup {
    maud::html! {
              (super::super::head(server))
        div class="hero min-h-screen bg-base-200" {
            div class="hero-content flex-col lg:flex-row-reverse" {
                div class="card shrink-0 w-full max-w-sm shadow-2xl bg-base-100" {
                    div class="card-body" {
                        div class="text-center mb-6" {
                            h2 class="text-2xl font-bold" {
                                "Sign in to your account"
                            }
                            p class="text-base-content/70 mt-2" {
                                "Password authentication has been disabled. Please use SSO to sign in."
                            }
                            @if let Some(msg) = message {
                                div class="alert alert-error mt-4" {
                                    (msg)
                                }
                            }
                        }

                        div class="form-control" {
                            a href="/auth/oidc/login" class="btn btn-primary w-full" {
                                "Sign in with SSO"
                            }
                        }
                    }
                }
            }
        }
    }
}
