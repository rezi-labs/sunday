use crate::db::user::Model as UserModel;
use crate::user::User;
use maud::{Markup, html};

/// User dashboard view with wider layout and direct edit button
pub fn dashboard(user: &UserModel, _current_user: &User) -> Markup {
    dashboard_with_message(user, _current_user, None)
}

/// User dashboard view with optional success message
pub fn dashboard_with_message(
    user: &UserModel,
    _current_user: &User,
    message: Option<&str>,
) -> Markup {
    html! {
        // Success message if provided
        @if let Some(msg) = message {
            div class="container mx-auto px-4 pt-4 max-w-6xl" {
                div class="alert alert-success mb-4" {
                    svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24" {
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z";
                    }
                    span { (msg) }
                }
            }
        }

        // Hero section
        div class="hero bg-base-100 shadow-sm mb-8" {
            div class="hero-content text-center py-8" {
                div class="max-w-md" {
                    h1 class="text-4xl font-bold text-primary" { "My Dashboard" }
                    p class="py-2 text-base-content/70" { "Manage your profile and account settings" }
                }
            }
        }

        // Main content area with wider layout
        div id="dashboard-content" class="container mx-auto px-4 py-8 max-w-6xl" {
            div class="grid grid-cols-1 lg:grid-cols-3 gap-8" {
                // Left column - Profile Card
                div class="lg:col-span-2" {
                    div class="card bg-base-100 shadow-xl" {
                        div class="card-body" {
                            div class="flex justify-between items-start mb-6" {
                                div {
                                    h2 class="card-title text-2xl mb-2" { "Profile Information" }
                                    p class="text-base-content/70" { "Your account details and settings" }
                                }
                                // Direct edit button (no three dots)
                                button hx-get="/dashboard/edit"
                                       hx-target="#dashboard-content"
                                       hx-swap="innerHTML"
                                       class="btn btn-primary gap-2" {
                                    svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z";
                                    }
                                    "Edit Profile"
                                }
                            }

                            div id="dashboard-content" {
                                div class="grid grid-cols-1 md:grid-cols-2 gap-6" {
                                    // Username
                                    div class="form-control" {
                                        label class="label" {
                                            span class="label-text font-semibold" { "Username" }
                                        }
                                        div class="input input-bordered flex items-center bg-base-200" {
                                            svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-70 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z";
                                            }
                                            span class="flex-grow" { (user.username) }
                                        }
                                    }

                                    // Email
                                    div class="form-control" {
                                        label class="label" {
                                            span class="label-text font-semibold" { "Email" }
                                        }
                                        div class="input input-bordered flex items-center bg-base-200" {
                                            svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-70 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 4.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z";
                                            }
                                            span class="flex-grow font-mono text-sm" { (user.email) }
                                        }
                                    }

                                    // Account Status
                                    div class="form-control" {
                                        label class="label" {
                                            span class="label-text font-semibold" { "Account Status" }
                                        }
                                        div class="flex items-center gap-2" {
                                            @if user.is_active {
                                                div class="badge badge-success gap-1" {
                                                    div class="w-2 h-2 bg-success-content rounded-full animate-pulse" {}
                                                    "Active"
                                                }
                                            } @else {
                                                div class="badge badge-error" { "Inactive" }
                                            }
                                            @if user.is_admin {
                                                div class="badge badge-primary gap-1" {
                                                    svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.031 9-11.622 0-1.042-.133-2.052-.382-3.016z";
                                                    }
                                                    "Administrator"
                                                }
                                            }
                                        }
                                    }

                                    // Account Role
                                    div class="form-control" {
                                        label class="label" {
                                            span class="label-text font-semibold" { "Role" }
                                        }
                                        div class="input input-bordered flex items-center bg-base-200" {
                                            svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-70 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z";
                                            }
                                            span class="flex-grow" {
                                                @if user.is_admin { "Administrator" } @else { "User" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Right column - Account Info and Stats
                div class="space-y-6" {
                    // Account Information Card
                    div class="card bg-base-100 shadow-xl" {
                        div class="card-body" {
                            h3 class="card-title text-lg mb-4" { "Account Information" }
                            div class="space-y-4" {
                                div {
                                    label class="label" {
                                        span class="label-text font-semibold" { "Member Since" }
                                    }
                                    div class="tooltip" data-tip=(user.created_at.format("%B %d, %Y at %I:%M %p")) {
                                        p class="text-sm bg-base-200 rounded-lg px-3 py-2" {
                                            (user.created_at.format("%B %d, %Y"))
                                        }
                                    }
                                }
                                div {
                                    label class="label" {
                                        span class="label-text font-semibold" { "Last Login" }
                                    }
                                    @if let Some(last_login) = user.last_login_at {
                                        div class="tooltip" data-tip=(last_login.format("%B %d, %Y at %I:%M %p")) {
                                            p class="text-sm bg-base-200 rounded-lg px-3 py-2" {
                                                (last_login.format("%B %d, %Y"))
                                            }
                                        }
                                    } @else {
                                        p class="text-sm text-base-content/50 italic bg-base-200 rounded-lg px-3 py-2" {
                                            "Never logged in"
                                        }
                                    }
                                }
                                div {
                                    label class="label" {
                                        span class="label-text font-semibold" { "Profile Updated" }
                                    }
                                    @if let Some(updated_at) = user.updated_at {
                                        div class="tooltip" data-tip=(updated_at.format("%B %d, %Y at %I:%M %p")) {
                                            p class="text-sm bg-base-200 rounded-lg px-3 py-2" {
                                                (updated_at.format("%B %d, %Y"))
                                            }
                                        }
                                    } @else {
                                        p class="text-sm text-base-content/50 italic bg-base-200 rounded-lg px-3 py-2" {
                                            "Not updated"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Quick Actions Card
                    div class="card bg-base-100 shadow-xl" {
                        div class="card-body" {
                            h3 class="card-title text-lg mb-4" { "Quick Actions" }
                            div class="space-y-3" {
                            button hx-get="/dashboard/edit"
                                   hx-target="#dashboard-content"
                                   hx-swap="innerHTML"
                                   class="btn btn-outline btn-sm w-full justify-start gap-2" {
                                svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z";
                                }
                                "Edit Profile"
                            }

                            @if user.is_admin {
                                    a href="/admin" class="btn btn-outline btn-sm w-full justify-start gap-2" {
                                        svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z";
                                        }
                                        "Administration"
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

/// User profile edit form with wider layout
pub fn edit_profile_form(user: &UserModel) -> Markup {
    html! {
        // Hero section
        div class="hero bg-base-100 shadow-sm mb-8" {
            div class="hero-content text-center py-8" {
                div class="max-w-md" {
                    h1 class="text-4xl font-bold text-primary" { "Edit Profile" }
                    p class="py-2 text-base-content/70" { "Update your account information" }
                }
            }
        }

        // Main content area with wider layout
        div class="container mx-auto px-4 py-8 max-w-6xl" {
            div class="card bg-base-100 shadow-xl" {
                div class="card-body" {
                    div class="mb-6" {
                        h2 class="card-title text-2xl" { "Profile Information" }
                    }

                    form id="profile-form"
                          hx-put="/dashboard/update"
                          hx-target="#dashboard-content"
                          hx-swap="innerHTML"
                          class="space-y-6" {

                        div class="grid grid-cols-1 md:grid-cols-2 gap-6" {
                            // Username field
                            div class="form-control" {
                                label class="label" {
                                    span class="label-text font-semibold" { "Username" }
                                }
                                div class="input-group" {
                                    span class="bg-base-200 border-base-300" {
                                        svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z";
                                        }
                                    }
                                    input type="text"
                                           name="username"
                                           value=(user.username)
                                           class="input input-bordered flex-1"
                                           required;
                                }
                            }

                            // Email field
                            div class="form-control" {
                                label class="label" {
                                    span class="label-text font-semibold" { "Email" }
                                }
                                div class="input-group" {
                                    span class="bg-base-200 border-base-300" {
                                        svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 4.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z";
                                        }
                                    }
                                    input type="email"
                                           name="email"
                                           value=(user.email)
                                           class="input input-bordered flex-1"
                                           required;
                                }
                            }
                        }

                        // Read-only fields
                        div class="border-t border-base-300 pt-6" {
                            h3 class="text-lg font-semibold mb-4" { "Account Information" }
                            div class="grid grid-cols-1 md:grid-cols-2 gap-6" {
                                div class="form-control" {
                                    label class="label" {
                                        span class="label-text font-semibold" { "Account Status" }
                                    }
                                    div class="flex items-center gap-2" {
                                        @if user.is_active {
                                            div class="badge badge-success gap-1" {
                                                div class="w-2 h-2 bg-success-content rounded-full animate-pulse" {}
                                                "Active"
                                            }
                                        } @else {
                                            div class="badge badge-error" { "Inactive" }
                                        }
                                        @if user.is_admin {
                                            div class="badge badge-primary gap-1" {
                                                svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.031 9-11.622 0-1.042-.133-2.052-.382-3.016z";
                                                }
                                                "Administrator"
                                            }
                                        }
                                    }
                                }

                                div class="form-control" {
                                    label class="label" {
                                        span class="label-text font-semibold" { "Member Since" }
                                    }
                                    div class="input input-bordered bg-base-200" {
                                        (user.created_at.format("%B %d, %Y"))
                                    }
                                }
                            }
                        }

                        // Action buttons
                        div class="card-actions justify-end pt-6 border-t border-base-300" {
                            button type="submit"
                                   class="btn btn-primary gap-2"
                                   hx-indicator="#loading" {
                                svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7";
                                }
                                "Save Changes"
                            }
                            // Loading indicator
                            span id="loading" class="loading loading-spinner loading-sm htmx-indicator" {}
                        }
                    }
                }
            }
        }
    }
}

/// Chat page view
pub fn chat_page(tenant: &str, server: &crate::config::Server, user: &crate::user::User) -> Markup {
    let chat_url = format!("/{}/api/chat", tenant);

    html! {
        // Hero section
        div class="hero bg-base-100 shadow-sm mb-8" {
            div class="hero-content text-center py-8" {
                div class="max-w-md" {
                    h1 class="text-4xl font-bold text-primary" { "AI Chat" }
                    p class="py-2 text-base-content/70" { "Chat with " (server.sunday_name()) " AI Assistant" }
                }
            }
        }

        // Main chat area
        div class="container mx-auto px-4 py-8 max-w-6xl" {
            div class="card bg-base-100 shadow-xl" {
                div class="card-body p-2" {
                    // Deep Chat Component
                    deep-chat
                        style="width: 100%; height: 600px; border-radius: 8px;"
                        connect=(format!(r#"{{"url": "{}", "method": "POST"}}"#, chat_url))
                        intro-message=(format!(r#"{{"text": "Hello {}! I'm {} AI. How can I help you today?"}}"#, user.user_name(), server.sunday_name()))
                    {}
                }
            }
        }
    }
}
