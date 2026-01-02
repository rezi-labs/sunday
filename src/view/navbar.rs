use maud::{Markup, html};

use crate::config::Server;
use crate::user::User;
use crate::view::icons::{info_icon, sunday_icon, theme_icon};

pub fn render(user: Option<&User>, server: &Server, tenant: Option<&str>) -> Markup {
    html! {
       (navbar(user, server, tenant))
    }
}

fn navbar(user: Option<&User>, server: &Server, tenant: Option<&str>) -> Markup {
    // Build tenant-aware URLs - ensure we have a tenant path
    let base_path = if let Some(t) = tenant {
        format!("/{}", t)
    } else if server.local() {
        "/acme".to_string()
    } else {
        "".to_string()
    };

    html! {
        nav class="bg-base-100 border-b border-base-200 px-6 py-4" {
            div class="max-w-[80rem] mx-auto flex items-center justify-between" {
                div class="flex items-center" {
                    a href=(if tenant.is_some() { format!("{}/dashboard", &base_path) } else { "/".to_string() }) class="flex items-center gap-3 hover:opacity-80 transition-opacity" {
                        span class="w-5 h-5 flex items-center justify-center opacity-70" {
                            (sunday_icon())
                        }
                        h1 class="text-lg font-light tracking-widest uppercase text-base-content" {
                            (server.sunday_name())
                            @if let Some(_t) = tenant {
                                span class="text-sm font-normal text-base-content/60 ml-2" {
                                    "(" (_t) ")"
                                }
                            }
                        }
                    }
                }

                div class="flex items-center gap-4" {
                    // Navigation links
                    (navigation_links(user, &base_path))

                    // Dropdown menu
                    (burger_menu(user, &base_path))
                }
            }
        }
    }
}

fn navigation_links(user: Option<&User>, base_path: &str) -> Markup {
    html! {
        div class="hidden md:flex items-center gap-6" {
            @if user.is_some() {
                a href=(format!("{}/chat", base_path)) class="text-base-content/70 hover:text-base-content transition-colors" {
                    "Chat"
                }
            }
            a href=(format!("{}/about", base_path)) class="text-base-content/70 hover:text-base-content transition-colors" {
                "About"
            }
            @if user.is_some() {
                @if user.unwrap().is_admin() {
                    // Admin links can be added here if needed
                }
            }
        }
    }
}

fn burger_menu(user: Option<&User>, base_path: &str) -> Markup {
    html! {
        div class="dropdown dropdown-end" {
            div tabindex="0" role="button" class="btn btn-ghost btn-square" {
                svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 6.75a.75.75 0 110-1.5.75.75 0 010 1.5zM12 12.75a.75.75 0 110-1.5.75.75 0 010 1.5zM12 18.75a.75.75 0 110-1.5.75.75 0 010 1.5z" {}
                }
            }
            ul tabindex="0" class="dropdown-content menu bg-base-100 z-[1] w-52 p-2 shadow-lg border border-base-200 rounded-lg mt-2" {
                // Mobile navigation links (visible on mobile only)
                div class="md:hidden" {
                    @if user.is_some() {
                        li {
                            a href=(format!("{}/chat", base_path)) class="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-base-200 transition-colors" {
                                span class="w-4 h-4 flex items-center opacity-60" {
                                    svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" {}
                                    }
                                }
                                "Chat"
                            }
                        }
                    }
                    li {
                        a href=(format!("{}/about", base_path)) class="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-base-200 transition-colors" {
                            span class="w-4 h-4 flex items-center opacity-60" {
                                (info_icon())
                            }
                            "About"
                        }
                    }
                    @if user.is_some() {
                        @if user.unwrap().is_admin() {
                            // Admin links can be added here if needed
                        }
                    }
                    div class="divider my-1" {}
                }

                li {
                    button onclick="toggleTheme()" class="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-base-200 transition-colors w-full text-left bg-transparent border-none cursor-pointer" {
                        span class="w-4 h-4 flex items-center opacity-60" {
                            (theme_icon())
                        }
                        "Theme"
                    }
                }
                @if user.is_none() {
                    li {
                        a href=(format!("{}auth", base_path)) class="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-primary hover:text-primary-content transition-colors text-primary font-medium" {
                            span class="w-4 h-4 flex items-center opacity-60" {
                                svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" {}
                                }
                            }
                            "Login"
                        }
                    }
                } @else {
                    li {
                        a href=(format!("{}dashboard", base_path)) class="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-base-200 transition-colors" {
                            span class="w-4 h-4 flex items-center opacity-60" {
                                svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" {}
                                }
                            }
                             "Profile"
                        }
                    }
                    @if user.unwrap().is_admin() {
                        li {
                            a href=(format!("{}admin", base_path)) class="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-base-200 transition-colors" {
                                svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065z" {}
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" {}
                                }
                                "Administration"
                            }
                        }
                    }
                    li {
                        a href=(format!("{}auth/logout", base_path)) class="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-base-200 transition-colors" {
                            span class="w-4 h-4 flex items-center opacity-60" {
                                svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" {}
                                }
                            }
                            "Logout"
                        }
                    }
                }
            }
        }
    }
}
