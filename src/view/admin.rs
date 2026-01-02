use crate::db::theme::Theme;
use crate::db::user::Model as UserModel;
use crate::user::User;
use crate::view::navbar;

pub fn dashboard(
    users: Vec<UserModel>,
    current_user: Option<&User>,
    search_term: Option<&str>,
) -> maud::Markup {
    maud::html! {
        (super::head())
        body class="bg-base-100 min-h-screen" {
            (navbar::render(current_user))

            // Main content area with better spacing
            div class="container mx-auto px-4 py-8 max-w-7xl" {
                // Statistics cards
                div class="stats shadow mb-8 w-full" {
                    div class="stat" {
                        div class="stat-title" { "Total Users" }
                        div class="stat-value text-primary" { (users.len()) }
                        div class="stat-desc" { "Registered accounts" }
                    }
                    div class="stat" {
                        div class="stat-title" { "Active Users" }
                        div class="stat-value text-success" {
                            (users.iter().filter(|u| u.is_active).count())
                        }
                        div class="stat-desc" { "Currently active" }
                    }
                    div class="stat" {
                        div class="stat-title" { "Administrators" }
                        div class="stat-value text-warning" {
                            (users.iter().filter(|u| u.is_admin).count())
                        }
                        div class="stat-desc" { "Admin accounts" }
                    }
                }

                // Action bar
                div class="navbar bg-base-100 rounded-box shadow mb-6" {
                    div class="navbar-start" {
                        div class="form-control relative" {
                            input type="text"
                                  name="search"
                                  placeholder="Search users... (Ctrl+K)"
                                  class="input input-bordered w-64 pr-10"
                                  id="user-search"
                                  hx-get="/admin"
                                  hx-target="body"
                                  hx-trigger="input changed delay:500ms, search"
                                  hx-indicator="#search-indicator"
                                  value=(search_term.unwrap_or(""));
                            div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none" {
                                span id="search-indicator" class="loading loading-spinner loading-sm htmx-indicator" {}
                                svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-base-content/40 htmx-hide-while-loading" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z";
                                }
                            }
                        }
                    }
                    div class="navbar-center" {
                        a href="/admin/themes" class="btn btn-outline gap-2" {
                            svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zM21 5H9M21 9H9M21 13H9M21 17H9";
                            }
                            "Theme Settings"
                        }
                    }
                    div class="navbar-end" {
                        button hx-get="/admin/users/create"
                               hx-target="#main-content"
                               hx-swap="innerHTML"
                               class="btn btn-primary gap-2" {
                            svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4";
                            }
                            "Create New User"
                        }
                    }
                }

                // Main content card
                div class="card bg-base-100 shadow-xl" {
                    div class="card-body p-0" {
                        div id="main-content" {
                            div class="overflow-x-auto" {
                                table class="table" {
                                    thead {
                                        tr class="border-b border-base-300" {
                                            th class="text-left font-semibold" {
                                                div class="flex items-center gap-2" {
                                                    "Username"
                                                    svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7l3-3 3 3m0 8l-3 3-3-3";
                                                    }
                                                }
                                            }
                                            th class="text-left font-semibold" { "Email" }
                                            th class="text-center font-semibold" { "Role" }
                                            th class="text-center font-semibold" { "Status" }
                                            th class="text-left font-semibold" { "Created" }
                                            th class="text-left font-semibold" { "Last Login" }
                                            th class="text-center font-semibold" { "Actions" }
                                        }
                                    }
                                    tbody {
                                        @if users.is_empty() {
                                            tr {
                                                td colspan="7" class="text-center py-8" {
                                                    div class="flex flex-col items-center gap-4" {
                                                        div class="w-16 h-16 rounded-full bg-base-200 flex items-center justify-center" {
                                                            @if search_term.is_some() {
                                                                svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-base-content/50" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z";
                                                                }
                                                            } @else {
                                                                svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-base-content/50" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z";
                                                                }
                                                            }
                                                        }
                                                        @if let Some(term) = search_term {
                                                            p class="text-base-content/70" { "No users match your search for \"" (term) "\"" }
                                                            button hx-get="/admin"
                                                                   hx-target="body"
                                                                   class="btn btn-ghost btn-sm" {
                                                                "Clear Search"
                                                            }
                                                        } @else {
                                                            p class="text-base-content/70" { "No users found" }
                                                            button hx-get="/admin/users/create"
                                                                   hx-target="#main-content"
                                                                   hx-swap="innerHTML"
                                                                   class="btn btn-primary btn-sm" {
                                                                "Create First User"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } @else {
                                            @for user in users {
                                                tr class="hover:bg-base-200/50 transition-colors" {
                                                    td {
                                                        div {
                                                            div class="font-semibold" { (user.username) }
                                                            @if user.is_admin {
                                                                div class="text-xs text-primary" { "Administrator" }
                                                            }
                                                        }
                                                    }
                                                    td class="font-mono text-sm" { (user.email) }
                                                    td class="text-center" {
                                                        @if user.is_admin {
                                                            div class="badge badge-primary badge-sm" {
                                                                svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.031 9-11.622 0-1.042-.133-2.052-.382-3.016z";
                                                                }
                                                                "Admin"
                                                            }
                                                        } @else {
                                                            div class="badge badge-ghost badge-sm" { "User" }
                                                        }
                                                    }
                                                    td class="text-center" {
                                                        @if user.is_active {
                                                            div class="badge badge-success badge-sm gap-1" {
                                                                div class="w-2 h-2 bg-success-content rounded-full animate-pulse" {}
                                                                "Active"
                                                            }
                                                        } @else {
                                                            div class="badge badge-error badge-sm" { "Inactive" }
                                                        }
                                                    }
                                                    td {
                                                        div class="tooltip" data-tip=(user.created_at.format("%B %d, %Y at %I:%M %p")) {
                                                            time class="text-sm" { (user.created_at.format("%Y-%m-%d")) }
                                                        }
                                                    }
                                                    td {
                                                        @if let Some(last_login) = user.last_login_at {
                                                            div class="tooltip" data-tip=(last_login.format("%B %d, %Y at %I:%M %p")) {
                                                                time class="text-sm" { (last_login.format("%Y-%m-%d")) }
                                                            }
                                                        } @else {
                                                            span class="text-base-content/50 text-sm italic" { "Never" }
                                                        }
                                                    }
                                                    td class="text-center" {
                                                        button hx-get=(format!("/admin/users/{}/edit", user.id))
                                                               hx-target="#main-content"
                                                               hx-swap="innerHTML"
                                                               class="btn btn-ghost btn-xs tooltip"
                                                               data-tip="Edit user" {
                                                            svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z";
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
                    }
                }
            }

            script {
                (maud::PreEscaped(r#"
            // Auto-refresh the dashboard every 30 seconds
            setInterval(function() {
                if (document.getElementById('main-content').querySelector('table')) {
                    htmx.trigger('#main-content', 'refresh');
                }
            }, 30000);

            // Search keyboard shortcut (Ctrl+K or Cmd+K)
            document.addEventListener('keydown', function(e) {
                if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                    e.preventDefault();
                    const searchInput = document.getElementById('user-search');
                    if (searchInput) {
                        searchInput.focus();
                    }
                }
            });

            // Enhanced loading states
            document.body.addEventListener('htmx:beforeRequest', function(evt) {
                if (evt.target.tagName === 'BUTTON') {
                    const button = evt.target;
                    button.classList.add('loading');
                    button.disabled = true;
                }
            });

            document.body.addEventListener('htmx:afterRequest', function(evt) {
                if (evt.target.tagName === 'BUTTON') {
                    const button = evt.target;
                    button.classList.remove('loading');
                    button.disabled = false;
                }
            });

            // Toast notifications for successful actions
            function showToast(message, type = 'success') {
                const toast = document.createElement('div');
                toast.className = `toast toast-top toast-end z-50`;
                toast.innerHTML = `
                    <div class="alert alert-${type} shadow-lg">
                        <div>
                            <span>${message}</span>
                        </div>
                    </div>
                `;
                document.body.appendChild(toast);

                setTimeout(() => {
                    toast.remove();
                }, 3000);
            }
             "#))
            }
        }
    }
}

pub fn theme_settings_page(
    themes: &[Theme],
    current_light_theme: &str,
    current_dark_theme: &str,
) -> maud::Markup {
    maud::html! {
        (super::head())
        body class="bg-base-100 min-h-screen" {
            (navbar::render(None::<&User>))

            div class="container mx-auto px-4 py-8 max-w-4xl" {
                div class="card bg-base-200 shadow-xl" {
                    div class="card-body" {
                        h1 class="card-title text-3xl mb-6" { "Theme Settings" }

                        form method="post" action="/admin/themes" class="space-y-6" {
                            // Default Light Theme
                            div class="form-control" {
                                label class="label" {
                                    span class="label-text font-semibold" { "Default Light Theme" }
                                }
                                select name="default_light_theme" class="select select-bordered w-full" required {
                                    @for theme in themes {
                                        option value=(theme.name) selected[theme.name == current_light_theme] {
                                            (theme.display_name)
                                            @if theme.is_custom { " (Custom)" }
                                        }
                                    }
                                }
                                label class="label" {
                                    span class="label-text-alt" { "Theme used when users have light mode enabled" }
                                }
                            }

                            // Default Dark Theme
                            div class="form-control" {
                                label class="label" {
                                    span class="label-text font-semibold" { "Default Dark Theme" }
                                }
                                select name="default_dark_theme" class="select select-bordered w-full" required {
                                    @for theme in themes {
                                        option value=(theme.name) selected[theme.name == current_dark_theme] {
                                            (theme.display_name)
                                            @if theme.is_custom { " (Custom)" }
                                        }
                                    }
                                }
                                label class="label" {
                                    span class="label-text-alt" { "Theme used when users have dark mode enabled" }
                                }
                            }

                            // Current Settings Display
                            div class="bg-base-100 rounded-lg p-4 mt-6" {
                                h3 class="font-semibold mb-2" { "Current Settings:" }
                                div class="space-y-1 text-sm" {
                                    div {
                                        span class="font-medium" { "Light Theme: " }
                                        span class="text-primary" { (current_light_theme) }
                                    }
                                    div {
                                        span class="font-medium" { "Dark Theme: " }
                                        span class="text-secondary" { (current_dark_theme) }
                                    }
                                }
                            }

                            // Action buttons
                            div class="flex gap-4 justify-end mt-8" {
                                a href="/admin" class="btn btn-outline" {
                                    "Back to Admin"
                                }
                                button type="submit" class="btn btn-primary" {
                                    "Update Theme Settings"
                                }
                            }
                        }

                        // Available Themes Info
                        div class="mt-8" {
                            h3 class="text-lg font-semibold mb-4" { "Available Themes" }
                            div class="overflow-x-auto" {
                                table class="table table-zebra w-full" {
                                    thead {
                                        tr {
                                            th { "Name" }
                                            th { "Display Name" }
                                            th { "Type" }
                                            th { "Description" }
                                        }
                                    }
                                    tbody {
                                        @for theme in themes {
                                            tr {
                                                td class="font-mono text-xs" { (theme.name) }
                                                td class="font-medium" { (theme.display_name) }
                                                td {
                                                    @if theme.is_custom {
                                                        span class="badge badge-secondary" { "Custom" }
                                                    } @else {
                                                        span class="badge badge-primary" { "Built-in" }
                                                    }
                                                }
                                                td class="text-sm" {
                                                    @if let Some(desc) = &theme.description {
                                                        (desc)
                                                    } @else {
                                                        span class="text-base-content/50" { "No description" }
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
            }
        }
    }
}

pub fn create_user_form() -> maud::Markup {
    maud::html! {
        div class="hero bg-base-100 shadow-sm mb-6" {
            div class="hero-content text-center py-6" {
                div class="max-w-md" {
                    h2 class="text-3xl font-bold text-primary" { "Create New User" }
                    p class="py-2 text-base-content/70" { "Add a new user account to the system" }
                }
            }
        }

        div class="container mx-auto px-4 max-w-2xl" {
            div class="card bg-base-100 shadow-xl" {
                div class="card-body" {
                    form hx-post="/admin/users/create"
                         hx-target="#create-result"
                         hx-swap="innerHTML"
                         hx-indicator="#loading-indicator"
                         class="space-y-6" {

                        div class="grid grid-cols-1 md:grid-cols-2 gap-6" {
                            fieldset class="fieldset" {
                                legend class="fieldset-legend" { "Username" }
                                input id="username"
                                      name="username"
                                      type="text"
                                      required
                                      minlength="3"
                                      maxlength="50"
                                      class="input"
                                      placeholder="johndoe"
                                      autocomplete="username";
                                p class="label" { "Unique identifier for the user (3-50 characters)" }
                            }

                            fieldset class="fieldset" {
                                legend class="fieldset-legend" { "Email Address" }
                                input id="email"
                                      name="email"
                                      type="email"
                                      required
                                      class="input"
                                      placeholder="john@example.com"
                                      autocomplete="email";
                                p class="label" { "Used for login and notifications" }
                            }
                        }

                        div class="divider" { "Account Setup" }

                        div class="alert alert-info" {
                            div class="flex items-start gap-3" {
                                svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 w-6 h-6 mt-0.5" fill="none" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z";
                                }
                                div {
                                    h4 class="font-semibold" { "Password Generation" }
                                    p class="text-sm mt-1" { "A secure 12-character password will be automatically generated and displayed after account creation. Make sure to share it securely with the user." }
                                }
                            }
                        }

                        div class="alert alert-warning" {
                            div class="flex items-start gap-3" {
                                svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 w-6 h-6 mt-0.5" fill="none" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L3.314 16.5c-.77.833.192 2.5 1.732 2.5z";
                                }
                                div {
                                    h4 class="font-semibold" { "Security Notice" }
                                    p class="text-sm mt-1" { "New users will be created with standard user permissions. Admin privileges can be granted later if needed." }
                                }
                            }
                        }

                        div class="card-actions justify-end pt-4" {
                            button type="submit" class="btn btn-primary gap-2 min-w-32" {
                                span id="loading-indicator" class="loading loading-spinner loading-sm htmx-indicator" {}
                                svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" id="create-icon" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4";
                                }
                                span id="button-text" { "Create User" }
                            }
                        }
                    }

                    div id="create-result" class="mt-6" {
                        // Results will be displayed here
                    }
                }
            }
        }
    }
}

pub fn user_created_success(username: &str, email: &str, password: &str) -> maud::Markup {
    maud::html! {
        div class="space-y-6" {
            // Success notification
            div class="alert alert-success shadow-lg" {
                div class="flex items-start gap-4" {
                    div class="bg-success rounded-full p-2" {
                        svg xmlns="http://www.w3.org/2000/svg" class="stroke-current text-success-content h-6 w-6" fill="none" viewBox="0 0 24 24" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z";
                        }
                    }
                    div class="flex-1" {
                        h3 class="font-bold text-lg" { "User Account Created Successfully!" }
                        p class="text-sm mt-1 opacity-90" { "The new user account has been added to the system." }
                    }
                }
            }

            // User details card
            div class="card bg-base-100 shadow-lg border border-success/20" {
                div class="card-body" {
                    h4 class="card-title text-primary flex items-center gap-2" {
                        svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z";
                        }
                        "Account Details"
                    }

                    div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4" {
                        div class="stat bg-base-200/50 rounded-lg" {
                            div class="stat-title text-xs" { "Username" }
                            div class="stat-value text-lg text-primary" { (username) }
                            div class="stat-desc" { "User identifier" }
                        }

                        div class="stat bg-base-200/50 rounded-lg" {
                            div class="stat-title text-xs" { "Email Address" }
                            div class="stat-value text-lg text-primary" { (email) }
                            div class="stat-desc" { "Login email" }
                        }
                    }

                    div class="divider" { "Temporary Password" }

                    div class="bg-neutral/10 rounded-lg p-4 border-2 border-dashed border-neutral/30" {
                        div class="flex items-center justify-between" {
                            div class="flex items-center gap-3" {
                                svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-neutral-content" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-3a1 1 0 011-1h2.586l6.414-6.414a6 6 0 0111.7-2.586z";
                                }
                                div {
                                    p class="font-semibold" { "Generated Password" }
                                    p class="text-xs text-base-content/60" { "12-character secure password" }
                                }
                            }
                            button class="btn btn-ghost btn-sm gap-2" onclick="copyToClipboard(this)" data-password=(password) {
                                svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z";
                                }
                                "Copy"
                            }
                        }
                        div class="mt-3 p-3 bg-base-200 rounded font-mono text-center text-lg tracking-wider select-all" {
                            (password)
                        }
                    }
                }
            }

            // Important security notice
            div class="alert alert-warning shadow-lg" {
                div class="flex items-start gap-4" {
                    div class="bg-warning rounded-full p-2" {
                        svg xmlns="http://www.w3.org/2000/svg" class="stroke-current text-warning-content h-6 w-6" fill="none" viewBox="0 0 24 24" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L3.314 16.5c-.77.833.192 2.5 1.732 2.5z";
                        }
                    }
                    div {
                        h4 class="font-bold" { "Security Notice" }
                        div class="text-sm mt-2 space-y-1" {
                            p { "• This password will only be shown once - make sure to copy it" }
                            p { "• Share this password with the user through a secure channel" }
                            p { "• The user should change this password on first login" }
                            p { "• The account has standard user permissions by default" }
                        }
                    }
                }
            }

            // Action buttons
            div class="flex flex-col sm:flex-row gap-3 pt-4" {
                button hx-get="/admin/users/create"
                       hx-target="#main-content"
                       hx-swap="innerHTML"
                       class="btn btn-secondary gap-2 flex-1" {
                    svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4";
                    }
                    "Create Another User"
                }
            }
        }

        script {
            (maud::PreEscaped(r#"
            function copyToClipboard(button) {
                const password = button.getAttribute('data-password');
                navigator.clipboard.writeText(password).then(function() {
                    // Temporarily change button text
                    const originalContent = button.innerHTML;
                    button.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path></svg>Copied!';
                    button.classList.add('btn-success');
                    setTimeout(function() {
                        button.innerHTML = originalContent;
                        button.classList.remove('btn-success');
                    }, 2000);
                }).catch(function(err) {
                    console.error('Could not copy text: ', err);
                });
            }
            "#))
        }
    }
}

pub fn user_table_only(users: Vec<UserModel>) -> maud::Markup {
    maud::html! {
        div class="overflow-x-auto" {
            table class="table" {
                thead {
                    tr class="border-b border-base-300" {
                        th class="text-left font-semibold" {
                            div class="flex items-center gap-2" {
                                "Username"
                                svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7l3-3 3 3m0 8l-3 3-3-3";
                                }
                            }
                        }
                        th class="text-left font-semibold" { "Email" }
                        th class="text-center font-semibold" { "Role" }
                        th class="text-center font-semibold" { "Status" }
                        th class="text-left font-semibold" { "Created" }
                        th class="text-left font-semibold" { "Last Login" }
                        th class="text-center font-semibold" { "Actions" }
                    }
                }
                tbody {
                    @if users.is_empty() {
                        tr {
                            td colspan="7" class="text-center py-8" {
                                 div class="flex flex-col items-center gap-4" {
                                     div class="w-16 h-16 rounded-full bg-base-200 flex items-center justify-center" {
                                         svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-base-content/50" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                             path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z";
                                         }
                                     }
                                     p class="text-base-content/70" { "No users found" }
                                     button hx-get="/admin/users/create"
                                            hx-target="#main-content"
                                            hx-swap="innerHTML"
                                            class="btn btn-primary btn-sm" {
                                         "Create First User"
                                     }
                                }
                            }
                        }
                    } @else {
                        @for user in users {
                            tr class="hover:bg-base-200/50 transition-colors" {
                                td {
                                     div {
                                         div class="font-semibold" { (user.username) }
                                         @if user.is_admin {
                                             div class="text-xs text-primary" { "Administrator" }
                                         }
                                     }
                                }
                                td class="font-mono text-sm" { (user.email) }
                                td class="text-center" {
                                    @if user.is_admin {
                                        div class="badge badge-primary badge-sm" {
                                            svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 mr-1" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.031 9-11.622 0-1.042-.133-2.052-.382-3.016z";
                                            }
                                            "Admin"
                                        }
                                    } @else {
                                        div class="badge badge-ghost badge-sm" { "User" }
                                    }
                                }
                                td class="text-center" {
                                    @if user.is_active {
                                        div class="badge badge-success badge-sm gap-1" {
                                            div class="w-2 h-2 bg-success-content rounded-full animate-pulse" {}
                                            "Active"
                                        }
                                    } @else {
                                        div class="badge badge-error badge-sm" { "Inactive" }
                                    }
                                }
                                td {
                                    div class="tooltip" data-tip=(user.created_at.format("%B %d, %Y at %I:%M %p")) {
                                        time class="text-sm" { (user.created_at.format("%Y-%m-%d")) }
                                    }
                                }
                                td {
                                    @if let Some(last_login) = user.last_login_at {
                                        div class="tooltip" data-tip=(last_login.format("%B %d, %Y at %I:%M %p")) {
                                            time class="text-sm" { (last_login.format("%Y-%m-%d")) }
                                        }
                                    } @else {
                                        span class="text-base-content/50 text-sm italic" { "Never" }
                                    }
                                }
                                td class="text-center" {
                                    button hx-get=(format!("/admin/users/{}/edit", user.id))
                                           hx-target="#main-content"
                                           hx-swap="innerHTML"
                                           class="btn btn-ghost btn-xs tooltip"
                                           data-tip="Edit user" {
                                        svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z";
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
}

pub fn edit_user_form(user: &UserModel) -> maud::Markup {
    maud::html! {
        div class="hero bg-base-100 shadow-sm mb-6" {
            div class="hero-content text-center py-6" {
                div class="max-w-md" {
                    h2 class="text-3xl font-bold text-primary" { "Edit User" }
                    p class="py-2 text-base-content/70" { "Update user account information" }
                }
            }
        }

        div class="container mx-auto px-4 max-w-2xl" {
            div class="card bg-base-100 shadow-xl" {
                div class="card-body" {
                    form hx-put=(format!("/admin/users/{}", user.id))
                         hx-target="#main-content"
                         hx-swap="innerHTML"
                         hx-indicator="#loading-indicator"
                         class="space-y-6" {

                        div class="grid grid-cols-1 md:grid-cols-2 gap-6" {
                            fieldset class="fieldset" {
                                legend class="fieldset-legend" { "Username" }
                                input id="username"
                                      name="username"
                                      type="text"
                                      required
                                      minlength="3"
                                      maxlength="50"
                                      class="input"
                                      value=(user.username)
                                      autocomplete="username";
                                p class="label" { "Unique identifier for the user (3-50 characters)" }
                            }

                            fieldset class="fieldset" {
                                legend class="fieldset-legend" { "Email Address" }
                                input id="email"
                                      name="email"
                                      type="email"
                                      required
                                      class="input"
                                      value=(user.email)
                                      autocomplete="email";
                                p class="label" { "Used for login and notifications" }
                            }
                        }

                        div class="divider" { "Account Info" }

                        // Current user information
                        div class="grid grid-cols-1 md:grid-cols-3 gap-4" {
                            div class="stat bg-base-200/50 rounded-lg" {
                                div class="stat-title text-xs" { "Role" }
                                div class="stat-value text-sm" {
                                    @if user.is_admin {
                                        span class="badge badge-primary" { "Administrator" }
                                    } @else {
                                        span class="badge badge-ghost" { "User" }
                                    }
                                }
                            }

                            div class="stat bg-base-200/50 rounded-lg" {
                                div class="stat-title text-xs" { "Status" }
                                div class="stat-value text-sm" {
                                    @if user.is_active {
                                        span class="badge badge-success" { "Active" }
                                    } @else {
                                        span class="badge badge-error" { "Inactive" }
                                    }
                                }
                            }

                            div class="stat bg-base-200/50 rounded-lg" {
                                div class="stat-title text-xs" { "Created" }
                                div class="stat-value text-sm" { (user.created_at.format("%Y-%m-%d")) }
                            }
                        }

                        div class="alert alert-info" {
                            div class="flex items-start gap-3" {
                                svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 w-6 h-6 mt-0.5" fill="none" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z";
                                }
                                div {
                                    h4 class="font-semibold" { "Note" }
                                    p class="text-sm mt-1" { "This form only updates basic profile information. Use the dropdown actions in the user table to modify role and status." }
                                }
                            }
                        }

                        div class="card-actions justify-end pt-4" {
                            button type="submit" class="btn btn-primary gap-2 min-w-32" {
                                span id="loading-indicator" class="loading loading-spinner loading-sm htmx-indicator" {}
                                svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7";
                                }
                                span { "Update User" }
                            }
                        }
                    }
                }
            }
        }
    }
}
