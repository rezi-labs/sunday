use actix_cors::Cors;
use actix_web::{
    App, HttpServer,
    middleware::Logger,
    web::{self},
};
use env_logger::Env;
use sqlx::postgres::PgPoolOptions;

mod api_key_middleware;
mod chat;
mod config;
mod documents;
mod migrations;
mod models;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    env_logger::init_from_env(
        Env::default().default_filter_or(&log_level),
    );

    let c = config::from_env();
    let bind = c.clone();

    // Run database migrations
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    if let Err(e) = migrations::run_migrations(&database_url).await {
        log::error!("Failed to run database migrations: {}", e);
        log::error!("Server startup aborted due to migration failure");
        std::process::exit(1);
    }

    // Create database pool for the application
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            log::info!("Database connection pool created successfully");
            pool
        }
        Err(e) => {
            log::error!("Failed to create database pool: {}", e);
            log::error!("Server startup aborted due to database connection failure");
            std::process::exit(1);
        }
    };

    let url = format!("http://{}:{}", c.host(), c.port());

    println!("{}", sunday_ascii_art());

    log::info!("Server starting at {url}");
    log::info!("API key authentication enabled");

    // Initialize AI models if Azure OpenAI is configured
    let ai_models = if let Some(azure_config) = c.azure_openai() {
        match models::AiModels::from_azure_config(azure_config) {
            Ok(models) => {
                log::info!("Azure OpenAI models initialized successfully");
                Some(models)
            }
            Err(e) => {
                log::error!("Failed to initialize Azure OpenAI models: {}", e);
                if !c.fake_ai() {
                    log::error!("Server startup aborted due to AI models initialization failure");
                    std::process::exit(1);
                }
                None
            }
        }
    } else {
        if !c.fake_ai() {
            log::warn!("No Azure OpenAI configuration found, but fake_ai is disabled");
            log::warn!("AI features will not work properly without configuration");
        }
        None
    };

    // Log CORS configuration once
    if c.cors_allowed_origins().contains(&"*".to_string()) {
        log::warn!("CORS configured to allow all origins - this should not be used in production");
    } else {
        for origin in c.cors_allowed_origins() {
            log::info!("CORS: allowing origin {}", origin);
        }
    }

    // Log AI models configuration once
    if ai_models.is_some() {
        log::info!("AI models configured and ready");
    }

    let server = HttpServer::new(move || {
        // Configure CORS
        let cors = if c.cors_allowed_origins().contains(&"*".to_string()) {
            Cors::permissive()
        } else {
            let mut cors = Cors::default()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::ACCEPT,
                    actix_web::http::header::CONTENT_TYPE,
                ])
                .max_age(3600);

            for origin in c.cors_allowed_origins() {
                cors = cors.allowed_origin(origin);
            }
            cors
        };

        let mut app = App::new()
            .wrap(cors)
            .wrap(Logger::default().exclude("/health"))
            .wrap(Logger::new("%a %{User-Agent}i").exclude("/health"))
            .wrap(api_key_middleware::ApiKeyAuth::new(c.api_keys().to_vec()))
            .app_data(web::Data::new(c.clone()))
            .app_data(web::Data::new(pool.clone()));

        // Add AI models to app data if available
        if let Some(models) = ai_models.clone() {
            app = app.app_data(web::Data::new(models));
        }

        app
            // Public routes (no auth required)
            .service(routes::technical::health)
            // API routes
            .service(
                web::scope("/api")
                    .service(chat::chat_endpoint)
                    .service(documents::create_document)
                    .service(documents::upload_document),
            )
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
