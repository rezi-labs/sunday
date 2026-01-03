use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::Env;

mod api_key_middleware;
mod chat;
mod config;
mod migrations;
mod models;
mod routes;
mod tenant;
mod tenant_middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let c = config::from_env();
    let bind = c.clone();

    // Run database migrations
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    if let Err(e) = migrations::run_migrations(&database_url).await {
        log::error!("Failed to run database migrations: {}", e);
        log::error!("Server startup aborted due to migration failure");
        std::process::exit(1);
    }

    let url = format!("http://{}:{}", c.host(), c.port());

    println!("{}", sunday_ascii_art());

    log::info!("Server starting at {url}");
    log::info!("API key authentication enabled");

    // Initialize AI models if Azure OpenAI is configured
    let ai_models = if let Some(azure_config) = c.azure_openai() {
        match models::AiModels::from_azure_config(azure_config) {
            Ok(models) => {
                log::info!("Azure OpenAI models initialized successfully");
                Some(std::sync::Arc::new(models))
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

    // Initialize tenant manager
    let tenant_manager = std::sync::Arc::new(tenant::TenantManager::new());

    // Auto-create 'acme' tenant in local mode
    if c.local() {
        log::info!("Local mode detected - auto-creating 'acme' tenant");
        match tenant_manager.create_tenant("acme").await {
            Ok(_) => {
                log::info!("Auto-created 'acme' tenant successfully");
            }
            Err(e) => {
                // Don't fail startup if acme tenant already exists
                if e.to_string().contains("already exists") {
                    log::info!("'acme' tenant already exists, skipping creation");
                } else {
                    log::error!("Failed to auto-create 'acme' tenant: {:?}", e);
                    log::error!("Server startup aborted due to tenant creation failure");
                    std::process::exit(1);
                }
            }
        }
    }

    let server = HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Logger::default().exclude("/health"))
            .wrap(Logger::new("%a %{User-Agent}i").exclude("/health"))
            .wrap(api_key_middleware::ApiKeyAuth::new(c.api_keys().to_vec()))
            .app_data(web::Data::new(c.clone()))
            .app_data(web::Data::new(tenant_manager.clone()));

        // Add AI models to app data if available
        if let Some(ref models) = ai_models {
            app = app.app_data(web::Data::new(models.clone()));
        }

        app
            // Public routes (no auth required)
            .service(routes::technical::health)
            // Global tenant management routes
            .service(routes::tenants::scope())
            // Tenant-specific API routes
            .service(web::scope("/{tenant}/api").service(chat::chat_endpoint))
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
