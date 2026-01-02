use crate::db::user::Model as UserModel;
use crate::user::User;
use crate::view::navbar;

pub fn dashboard(
    users: Vec<UserModel>,
    current_user: Option<&User>,
    search_term: Option<&str>,
) -> maud::Markup {
    maud::html! {
        (super::super::head())
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
                        // Theme management removed - themes now loaded from external API
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
