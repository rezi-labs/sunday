use crate::config::{DatabaseClient, Server};
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod session;
pub mod theme;
pub mod user;

pub async fn create_db_client(
    config: &Server,
) -> Result<(DatabaseClient, tokio::task::JoinHandle<()>), Box<dyn std::error::Error>> {
    // Create SSL connector
    let builder = SslConnector::builder(SslMethod::tls())?;
    let connector = MakeTlsConnector::new(builder.build());

    const MAX_RETRIES: u32 = 10;
    const RETRY_DELAY_MS: u64 = 500;

    let mut last_error = None;

    // Retry connection up to MAX_RETRIES times
    for attempt in 1..=MAX_RETRIES {
        log::info!("Attempting database connection (attempt {attempt} of {MAX_RETRIES})");

        match tokio_postgres::connect(&config.db_url(), connector.clone()).await {
            Ok((client, connection)) => {
                log::info!("Database connection established on attempt {attempt}");

                let client = Arc::new(Mutex::new(client));

                // Spawn the connection task
                let connection_handle = tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        log::error!("Database connection error: {e}");
                    }
                });

                return Ok((client, connection_handle));
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < MAX_RETRIES {
                    log::warn!(
                        "Database connection failed on attempt {}: {}. Retrying in {}ms...",
                        attempt,
                        last_error.as_ref().unwrap(),
                        RETRY_DELAY_MS
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                } else {
                    log::error!(
                        "Database connection failed after {} attempts: {}",
                        MAX_RETRIES,
                        last_error.as_ref().unwrap()
                    );
                }
            }
        }
    }

    // If we get here, all retries failed
    Err(Box::new(last_error.unwrap()))
}
