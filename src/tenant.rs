use crate::config::Server;
use crate::migrations;
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::Client;

pub type TenantDatabaseClient = Arc<Mutex<Client>>;
pub type TenantDatabaseConnections = Arc<Mutex<HashMap<String, TenantDatabaseClient>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantResponse {
    pub name: String,
    pub database_url: String,
    pub status: String,
}

pub struct TenantManager {
    config: Server,
    connections: TenantDatabaseConnections,
}

impl TenantManager {
    pub fn new(config: Server) -> Self {
        Self {
            config,
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_tenant(
        &self,
        tenant_name: &str,
    ) -> Result<CreateTenantResponse, Box<dyn std::error::Error>> {
        log::info!("Creating tenant: {}", tenant_name);

        // Validate tenant name (alphanumeric and underscores only)
        if !tenant_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(
                "Tenant name must contain only alphanumeric characters and underscores".into(),
            );
        }

        // Connect to PostgreSQL server to create database
        let server_db_url = format!(
            "postgresql://{}:{}@{}:{}/postgres",
            self.config.db_user(),
            self.config.db_password(),
            self.config.db_host(),
            self.config.db_port()
        );

        // Create SSL connector
        let builder = SslConnector::builder(SslMethod::tls())?;
        let connector = MakeTlsConnector::new(builder.build());

        let (admin_client, admin_connection) =
            tokio_postgres::connect(&server_db_url, connector.clone()).await?;

        // Spawn the admin connection task
        let _admin_connection_handle = tokio::spawn(async move {
            if let Err(e) = admin_connection.await {
                log::error!("Admin database connection error: {e}");
            }
        });

        // Check if database already exists
        let existing_db = admin_client
            .query_one(
                "SELECT 1 FROM pg_database WHERE datname = $1",
                &[&tenant_name],
            )
            .await;

        if existing_db.is_ok() {
            return Err(format!("Tenant '{}' already exists", tenant_name).into());
        }

        // Create the database
        let create_db_query = format!("CREATE DATABASE \"{}\"", tenant_name);
        admin_client.execute(&create_db_query, &[]).await?;

        log::info!("Database '{}' created successfully", tenant_name);

        // Connect to the new tenant database and run migrations
        let tenant_db_url = self.config.build_tenant_db_url(tenant_name);
        let (tenant_client, tenant_connection) =
            tokio_postgres::connect(&tenant_db_url, connector).await?;

        // Spawn the tenant connection task
        let _tenant_connection_handle = tokio::spawn(async move {
            if let Err(e) = tenant_connection.await {
                log::error!("Tenant database connection error: {e}");
            }
        });

        // Run migrations on the new tenant database
        log::info!("Running migrations for tenant: {}", tenant_name);
        match migrations::run_migrations(&tenant_client).await {
            Ok(_) => {
                log::info!(
                    "Migrations completed successfully for tenant: {}",
                    tenant_name
                );
            }
            Err(e) => {
                log::error!("Migration failed for tenant {}: {:?}", tenant_name, e);
                // Clean up by dropping the database if migrations failed
                let drop_db_query = format!("DROP DATABASE \"{}\"", tenant_name);
                if let Err(drop_err) = admin_client.execute(&drop_db_query, &[]).await {
                    log::error!(
                        "Failed to cleanup database after migration failure: {:?}",
                        drop_err
                    );
                }
                return Err(format!(
                    "Failed to run migrations for tenant '{}': {}",
                    tenant_name, e
                )
                .into());
            }
        }

        // Store the connection for future use
        let tenant_client_arc = Arc::new(Mutex::new(tenant_client));
        {
            let mut connections = self.connections.lock().await;
            connections.insert(tenant_name.to_string(), tenant_client_arc);
        }

        Ok(CreateTenantResponse {
            name: tenant_name.to_string(),
            database_url: tenant_db_url,
            status: "created".to_string(),
        })
    }

    pub async fn get_tenant_connection(
        &self,
        tenant_name: &str,
    ) -> Result<TenantDatabaseClient, Box<dyn std::error::Error>> {
        // Check if connection already exists
        {
            let connections = self.connections.lock().await;
            if let Some(client) = connections.get(tenant_name) {
                return Ok(client.clone());
            }
        }

        // Connection doesn't exist, try to create it
        let tenant_db_url = self.config.build_tenant_db_url(tenant_name);

        // Create SSL connector
        let builder = SslConnector::builder(SslMethod::tls())?;
        let connector = MakeTlsConnector::new(builder.build());

        // Check if database exists before connecting
        let server_db_url = format!(
            "postgresql://{}:{}@{}:{}/postgres",
            self.config.db_user(),
            self.config.db_password(),
            self.config.db_host(),
            self.config.db_port()
        );

        let (admin_client, admin_connection) =
            tokio_postgres::connect(&server_db_url, connector.clone()).await?;

        // Spawn the admin connection task
        let _admin_connection_handle = tokio::spawn(async move {
            if let Err(e) = admin_connection.await {
                log::error!("Admin database connection error: {e}");
            }
        });

        let existing_db = admin_client
            .query_one(
                "SELECT 1 FROM pg_database WHERE datname = $1",
                &[&tenant_name],
            )
            .await;

        if existing_db.is_err() {
            return Err(format!("Tenant '{}' does not exist", tenant_name).into());
        }

        // Connect to tenant database
        let (tenant_client, tenant_connection) =
            tokio_postgres::connect(&tenant_db_url, connector).await?;

        // Spawn the tenant connection task
        let _tenant_connection_handle = tokio::spawn(async move {
            if let Err(e) = tenant_connection.await {
                log::error!("Tenant database connection error: {e}");
            }
        });

        let tenant_client_arc = Arc::new(Mutex::new(tenant_client));

        // Store the connection
        {
            let mut connections = self.connections.lock().await;
            connections.insert(tenant_name.to_string(), tenant_client_arc.clone());
        }

        Ok(tenant_client_arc)
    }

    pub fn connections(&self) -> TenantDatabaseConnections {
        self.connections.clone()
    }

    pub async fn list_tenants(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let server_db_url = format!(
            "postgresql://{}:{}@{}:{}/postgres",
            self.config.db_user(),
            self.config.db_password(),
            self.config.db_host(),
            self.config.db_port()
        );

        // Create SSL connector
        let builder = SslConnector::builder(SslMethod::tls())?;
        let connector = MakeTlsConnector::new(builder.build());

        let (admin_client, admin_connection) =
            tokio_postgres::connect(&server_db_url, connector).await?;

        // Spawn the admin connection task
        let _admin_connection_handle = tokio::spawn(async move {
            if let Err(e) = admin_connection.await {
                log::error!("Admin database connection error: {e}");
            }
        });

        // Get list of databases that are not system databases
        let rows = admin_client
            .query(
                "SELECT datname FROM pg_database WHERE datname NOT IN ('postgres', 'template0', 'template1') ORDER BY datname",
                &[],
            )
            .await?;

        let tenants: Vec<String> = rows.iter().map(|row| row.get("datname")).collect();
        Ok(tenants)
    }
}
