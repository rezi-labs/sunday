use crate::oidc::OidcConfig;
use actix_web::cookie::Key;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::Client;

#[derive(Clone)]
pub struct Server {
    port: u16,
    host: String,
    db_url: String,
    local: bool,
    session_key: Key,
    admin_username: String,
    admin_password: String,
    reset_admin_user: bool,
    oidc_config: OidcConfig,
    pub pg_pool: PgPool,
    theme_api_url: Option<String>,
    sunday_name: String,
}

pub type DatabaseClient = Arc<Mutex<Client>>;

impl Server {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn db_url(&self) -> String {
        self.db_url.clone()
    }

    pub fn local(&self) -> bool {
        self.local
    }

    pub fn session_key(&self) -> Key {
        self.session_key.clone()
    }

    pub fn admin_username(&self) -> &str {
        &self.admin_username
    }

    pub fn admin_password(&self) -> &str {
        &self.admin_password
    }

    pub fn reset_admin_user(&self) -> bool {
        self.reset_admin_user
    }

    pub fn oidc_config(&self) -> &OidcConfig {
        &self.oidc_config
    }

    pub fn theme_api_url(&self) -> Option<&str> {
        self.theme_api_url.as_deref()
    }

    pub fn sunday_name(&self) -> &str {
        &self.sunday_name
    }
}

pub fn from_env() -> Server {
    let local = env::var("LOCAL").unwrap_or("false".to_string());
    let local = local == "true";

    let port: u16 = env::var("PORT")
        .or_else(|_| env::var("g_port"))
        .map(|e| e.parse().expect("could not parse port"))
        .unwrap_or(3000);

    let host: String = env::var("HOST")
        .or_else(|_| env::var("g_host"))
        .map(|e| e.parse().expect("could not parse host"))
        .unwrap_or("0.0.0.0".to_string());

    // Build database URL from individual components
    let db_user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let db_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let db_host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = env::var("POSTGRES_PORT")
        .unwrap_or_else(|_| "5500".to_string())
        .parse::<u16>()
        .expect("POSTGRES_PORT must be a valid port number");
    let db_name = env::var("POSTGRES_DB").unwrap_or_else(|_| "sunday".to_string());

    let db_url = format!("postgresql://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");

    // Generate session secret key
    let session_key = match env::var("SESSION_SECRET_KEY") {
        Ok(key_str) => {
            // Use provided secret key from environment
            let key_bytes = key_str.as_bytes();
            if key_bytes.len() >= 32 {
                log::info!("Using session secret from SESSION_SECRET_KEY environment variable");
                Key::from(key_bytes)
            } else {
                log::warn!("SESSION_SECRET_KEY is too short (< 32 bytes), generating random key");
                Key::generate()
            }
        }
        Err(_) => {
            log::warn!(
                "SESSION_SECRET_KEY not set, generating random key (sessions will not persist across restarts)"
            );
            log::info!("Set SESSION_SECRET_KEY environment variable for persistent sessions");
            log::info!("Generate with: openssl rand -base64 32");
            Key::generate()
        }
    };

    let admin_username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());

    let admin_password = env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set");

    let reset_admin_user = env::var("RESET_ADMIN_USER")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    let oidc_config = OidcConfig::from_env();

    // Theme API URL (optional)
    let theme_api_url = env::var("THEME_API_URL").ok();

    // Sunday name (with fallback)
    let sunday_name = env::var("SUNDAY_NAME").unwrap_or_else(|_| "Sunday".to_string());

    // Create PgPool for sqlx operations (will be initialized in main.rs)
    // For now, we'll use a placeholder that will be replaced in main.rs
    let pg_pool = PgPool::connect_lazy(&db_url).expect("Failed to create PgPool");

    Server {
        port,
        host,
        db_url,
        local,
        session_key,
        admin_username,
        admin_password,
        reset_admin_user,
        oidc_config,
        pg_pool,
        theme_api_url,
        sunday_name,
    }
}
