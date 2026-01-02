use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::Env;

mod admin_guard;
mod config;
mod db;
mod migrations;
mod oidc;
mod routes;
mod text_utils;
mod theme_service;
mod user;
mod view;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let c = config::from_env();
    let bind = c.clone();

    // Initialize database connection
    let (db_client, _connection_handle) = db::create_db_client(&c)
        .await
        .expect("Failed to create database connection");

    // Run migrations
    {
        let client = db_client.lock().await;
        match migrations::run_migrations(&client).await {
            Ok(_) => {
                log::info!("Database migrations completed successfully");
            }
            Err(e) => {
                log::error!("Database migrations failed: {e:?}");
                log::error!("Server startup aborted due to migration failure");
                std::process::exit(1);
            }
        }
    }
    // Ensure admin user exists
    match db::user::ensure_admin_user(
        c.admin_username(),
        c.admin_password(),
        &db_client,
        c.reset_admin_user(),
    )
    .await
    {
        Ok(_) => {
            log::info!("Admin user initialization completed");
        }
        Err(e) => {
            log::error!("Admin user initialization failed: {e:?}");
            log::error!("Server startup aborted due to admin initialization failure");
            std::process::exit(1);
        }
    }

    let url = format!("http://{}:{}", c.host(), c.port());

    println!("{}", sunday_ascii_art());

    log::info!("Server starting at {url}");

    // Get session secret key from config
    let secret_key = c.session_key();

    // Initialize OIDC client
    let oidc_client = oidc::OidcClient::new(c.oidc_config().clone());
    let oidc_client_arc = std::sync::Arc::new(tokio::sync::Mutex::new(oidc_client));

    // Initialize theme service
    let theme_service = std::sync::Arc::new(theme_service::ThemeService::new(
        c.theme_api_url().map(|s| s.to_string()),
    ));

    log::info!(
        "Configuring session middleware - secure cookies: {}",
        !c.local()
    );

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default().exclude("/health"))
            .wrap(Logger::new("%a %{User-Agent}i").exclude("/health"))
            .wrap(user::UserExtractor)
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(!c.local()) // Set to true in production with HTTPS
                    .cookie_name("sunday_session".to_string())
                    .cookie_http_only(true)
                    .build(),
            )
            .app_data(web::Data::new(c.clone()))
            .app_data(web::Data::new(db_client.clone()))
            .app_data(web::Data::new(oidc_client_arc.clone()))
            .app_data(web::Data::new(theme_service.clone()))
            .service(
                web::scope("/auth")
                    .route("", web::get().to(routes::auth::login_form))
                    .route("/login", web::get().to(routes::auth::login_form))
                    .route("/logout", web::get().to(routes::auth::logout))
                    .route("/oidc/login", web::get().to(routes::auth::oidc_login))
                    .route("/oidc/callback", web::get().to(routes::auth::oidc_callback)),
            )
            .service(
                web::scope("/api")
                    .route(
                        "/themes/css",
                        web::get().to(routes::technical::serve_theme_css),
                    )
                    .route(
                        "/themes/refresh",
                        web::post().to(routes::technical::refresh_theme_cache),
                    ),
            )
            .service(
                web::scope("/admin")
                    .wrap(admin_guard::AdminGuard)
                    .route("", web::get().to(routes::admin::dashboard))
                    .route("/dashboard", web::get().to(routes::admin::dashboard))
                    .service(
                        web::scope("/users")
                            .route(
                                "/create",
                                web::get().to(routes::admin::users::create_user_form),
                            )
                            .route("", web::post().to(routes::admin::users::create_user))
                            .route(
                                "/{id}/edit",
                                web::get().to(routes::admin::users::edit_user_form),
                            )
                            .route("/{id}", web::put().to(routes::admin::users::update_user))
                            .route("/{id}", web::delete().to(routes::admin::users::delete_user))
                            .route(
                                "/{id}/toggle-active",
                                web::post().to(routes::admin::users::toggle_user_active),
                            )
                            .route(
                                "/{id}/toggle-admin",
                                web::post().to(routes::admin::users::toggle_user_admin),
                            ),
                    ),
            )
            .service(
                web::scope("/dashboard")
                    .route("", web::get().to(routes::user::dashboard))
                    .route("/edit", web::get().to(routes::user::edit_profile_form))
                    .route("/update", web::put().to(routes::user::update_profile)),
            )
            .service(routes::chat::scope())
            .service(view::index_route)
            .service(view::index_table_route)
            .service(view::about_endpoint)
            .service(view::about_readme_endpoint)
            .service(routes::technical::health)
            .service(routes::assets::scope())
    });
    server
        .bind((bind.host(), bind.port()))
        .expect("Could not bind server address")
        .run()
        .await
}

fn sunday_ascii_art() -> &'static str {
    r#"
    \   |   /          ███████╗██╗   ██╗███╗   ██╗██████╗  █████╗ ██╗   ██╗
  .-.\_)_(/,-.       ██╔════╝██║   ██║████╗  ██║██╔══██╗██╔══██╗╚██╗ ██╔╝
 /  _     _  \       ███████╗██║   ██║██╔██╗ ██║██║  ██║███████║ ╚████╔╝
(   o   o   )       ╚════██║██║   ██║██║╚██╗██║██║  ██║██╔══██║  ╚██╔╝
 >  \_-_/  <        ███████║╚██████╔╝██║ ╚████║██████╔╝██║  ██║   ██║
 \_       _/        ╚══════╝ ╚═════╝ ╚═╝  ╚═══╝╚═════╝ ╚═╝  ╚═╝   ╚═╝
   `-----'
    "#
}
