pub fn login_form(username_value: String, message: Option<&str>) -> maud::Markup {
    maud::html! {
              (super::super::head())
        div class="hero min-h-screen bg-base-200" {
            div class="hero-content flex-col lg:flex-row-reverse" {
                div class="card shrink-0 w-full max-w-sm shadow-2xl bg-base-100" {
                    div class="card-body" {
                        div class="text-center mb-6" {
                            h2 class="text-2xl font-bold" {
                                "Sign in to your account"
                            }
                            @if !username_value.is_empty() {
                                p class="text-base-content/70 mt-2" {
                                    "Welcome back! Please enter your password for " (username_value)
                                }
                            }
                            @if let Some(msg) = message {
                                @if msg.starts_with("Account created") {
                                    div class="alert alert-success mt-4" {
                                        (msg)
                                    }
                                } @else {
                                    div class="alert alert-error mt-4" {
                                        (msg)
                                    }
                                }
                            }
                        }
                        form hx-post="/auth/login" hx-target="body" hx-swap="outerHTML" {
                            fieldset class="fieldset" {
                                legend class="fieldset-legend" { "Username" }
                                input id="username" name="username" type="text" autocomplete="username" required
                                    class="input"
                                    placeholder="Username" value=(username_value);
                            }
                            fieldset class="fieldset" {
                                legend class="fieldset-legend" { "Password" }
                                input id="password" name="password" type="password" autocomplete="current-password" required
                                    class="input"
                                    placeholder="Password";
                            }
                            div class="form-control mt-6" {
                                button type="submit" class="btn btn-primary w-full" {
                                    "Sign in"
                                }
                            }

                            div class="divider" { "OR" }

                            div class="form-control" {
                                a href="/auth/oidc/login" class="btn btn-outline w-full" {
                                    "Sign in with SSO"
                                }
                            }

                            div class="text-center mt-4" {
                                a hx-get="/auth" hx-target="body" hx-swap="outerHTML" class="link link-primary" {
                                    "Use a different username"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
