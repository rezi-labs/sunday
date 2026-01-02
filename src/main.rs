use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::Env;

mod admin_guard;
mod authentication;
mod config;
mod db;
mod migrations;
mod oidc;
mod routes;
mod text_utils;
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
            .service(
                web::scope("/auth")
                    .route("", web::get().to(routes::auth::login_form))
                    .route("/login", web::get().to(routes::auth::login_form))
                    .route("/login", web::post().to(routes::auth::login))
                    .route("/logout", web::get().to(routes::auth::logout))
                    .route("/oidc/login", web::get().to(routes::auth::oidc_login))
                    .route("/oidc/callback", web::get().to(routes::auth::oidc_callback)),
            )
            .service(
                web::scope("/admin")
                    .wrap(admin_guard::AdminGuard)
                    .route("", web::get().to(routes::admin::admin_dashboard))
                    .route(
                        "/users/create",
                        web::get().to(routes::admin::create_user_form),
                    )
                    .route("/users/create", web::post().to(routes::admin::create_user))
                    .route(
                        "/users/{id}/edit",
                        web::get().to(routes::admin::edit_user_form),
                    )
                    .route("/users/{id}", web::put().to(routes::admin::update_user))
                    .route("/users/{id}", web::delete().to(routes::admin::delete_user))
                    .route(
                        "/users/{id}/toggle-active",
                        web::post().to(routes::admin::toggle_user_active),
                    )
                    .route(
                        "/users/{id}/toggle-admin",
                        web::post().to(routes::admin::toggle_user_admin),
                    ),
            )
            .service(
                web::scope("/dashboard")
                    .route("", web::get().to(routes::user::dashboard))
                    .route("/edit", web::get().to(routes::user::edit_profile_form))
                    .route("/update", web::put().to(routes::user::update_profile)),
            )
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
████████╗ █████╗ ███████╗████████╗███████╗
╚══██╔══╝██╔══██╗██╔════╝╚══██╔══╝██╔════╝
   ██║   ███████║███████╗   ██║   █████╗
   ██║   ██╔══██║╚════██║   ██║   ██╔══╝
   ██║   ██║  ██║███████║   ██║   ███████╗
   ╚═╝   ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝
    "#
}
