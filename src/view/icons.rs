use maud::{Markup, html};

#[allow(unused)]
pub fn list_icon() -> Markup {
    html! {
        svg class="w-full h-full" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
            path stroke-linecap="round" stroke-linejoin="round" d="M8.25 6.75h12M8.25 12h12m-12 5.25h12M3.75 6.75h.007v.008H3.75V6.75Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0ZM3.75 12h.007v.008H3.75V12Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Zm-.375 5.25h.007v.008H3.75v-.008Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z" {
            }
        }
    }
}

#[allow(unused)]
pub fn share_icon() -> Markup {
    html! {
        svg class="w-full h-full" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
            path stroke-linecap="round" stroke-linejoin="round" d="M7.217 10.907a2.25 2.25 0 1 0 0 2.186m0-2.186c.18.324.283.696.283 1.093s-.103.77-.283 1.093m0-2.186 9.566-5.314m-9.566 7.5 9.566 5.314m0 0a2.25 2.25 0 1 0 3.935 2.186 2.25 2.25 0 0 0-3.935-2.186Zm0-12.814a2.25 2.25 0 1 0 3.933-2.185 2.25 2.25 0 0 0-3.933 2.185Z" {
            }
        }
    }
}
#[allow(unused)]
pub fn delete_icon() -> Markup {
    html! {
        svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" {
            }
        }
    }
}
#[allow(unused)]
pub fn add_icon() -> Markup {
    html! {
        svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" {
            }
        }
    }
}
#[allow(unused)]
pub fn user_icon() -> Markup {
    html! {
        svg."size-6" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" stroke-width="1.5" stroke="currentColor" {
            path stroke-linejoin="round" stroke-linecap="round" d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z" {}
        }
    }
}

#[allow(unused)]
pub fn house_icon() -> Markup {
    html! {
        svg class="w-full h-full" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
            path stroke-linecap="round" stroke-linejoin="round" d="M8.25 21v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21m0 0h4.5V3.545M12.75 21h7.5V10.75M2.25 21h1.5m18 0h-18M2.25 9l4.5-1.636M18.75 3l-1.5.545m0 6.205 3 1m1.5.5-1.5-.5M6.75 7.364V3h-3v18m3-13.636 10.5-3.819" {
            }
        }
    }
}
#[allow(unused)]
pub fn export_icon() -> Markup {
    html! {
        svg class="w-full h-full" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
            path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m.75 12 3 3m0 0 3-3m-3 3v-6m-1.5-9H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" {
            }
        }
    }
}

#[allow(unused)]
pub fn wand_icon() -> Markup {
    html! {
        svg class="w-full h-full" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" {
            line x1="15" y1="15" x2="21" y2="21" {
            }
            path d="M9 6L10 8L12 8L10.5 9.5L11 12L9 11L7 12L7.5 9.5L6 8L8 8L9 6z" fill="currentColor" {
            }
            path d="M16 8L17 9L16 10L15 9L16 8z" fill="currentColor" {
            }
            path d="M18 5L18.5 5.5L18 6L17.5 5.5L18 5z" fill="currentColor" {
            }
            path d="M4 17L4.5 17.5L4 18L3.5 17.5L4 17z" fill="currentColor" {
            }
        }
    }
}

#[allow(unused)]
pub fn link_icon() -> Markup {
    html! {
        svg class="w-full h-full" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
            path stroke-linecap="round" stroke-linejoin="round" d="M13.19 8.688a4.5 4.5 0 0 1 1.242 7.244l-4.5 4.5a4.5 4.5 0 0 1-6.364-6.364l1.757-1.757m13.35-.622 1.757-1.757a4.5 4.5 0 0 0-6.364-6.364l-4.5 4.5a4.5 4.5 0 0 0 1.242 7.244" {
            }
        }
    }
}

pub fn info_icon() -> Markup {
    html! {
        svg class="w-full h-full" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
            path stroke-linecap="round" stroke-linejoin="round" d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" {
            }
        }
    }
}

pub fn sunday_icon() -> Markup {
    html! {
        svg class="w-full h-full sunday-icon" viewBox="0 0 32 32" xmlns="http://www.w3.org/2000/svg" {
            style {
                r#"
                .sunday-icon .bg { 
                    fill: currentColor; 
                }
                .sunday-icon .fg { 
                    fill: var(--base-100, #ffffff); 
                }
                .sunday-icon .accent { 
                    fill: var(--base-100, #ffffff); 
                    opacity: 0.8; 
                }
                [data-theme="swiss-dark"] .sunday-icon .fg,
                [data-theme="swiss-dark"] .sunday-icon .accent { 
                    fill: var(--base-content, #000000); 
                }
                "#
            }

            // Background square with sharp edges
            rect x="2" y="2" width="28" height="28" class="bg" stroke="none" {}

            // Inner "T" shape in negative space - angular and geometric
            g class="fg" {
                // Top horizontal bar of T
                rect x="6" y="6" width="20" height="4" {}
                // Vertical bar of T
                rect x="14" y="6" width="4" height="20" {}
            }

            // Additional geometric accent - small squares for texture
            g class="accent" {
                rect x="8" y="12" width="2" height="2" {}
                rect x="22" y="12" width="2" height="2" {}
                rect x="8" y="22" width="2" height="2" {}
                rect x="22" y="22" width="2" height="2" {}
            }
        }
    }
}

#[allow(unused)]
pub fn theme_icon() -> Markup {
    html! {
        svg class="w-full h-full" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" {
            path stroke-linecap="round" stroke-linejoin="round" d="M21.752 15.002A9.72 9.72 0 0 1 18 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 0 0 3 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 0 0 9.002-5.998Z" {
            }
        }
    }
}
