use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantResponse {
    pub name: String,
    pub status: String,
}

pub struct TenantManager {
    tenants: Arc<Mutex<HashMap<String, CreateTenantResponse>>>,
}

impl TenantManager {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_tenant(
        &self,
        tenant_name: &str,
    ) -> Result<CreateTenantResponse, Box<dyn std::error::Error>> {
        log::info!("Creating tenant (fake): {}", tenant_name);

        // Validate tenant name (alphanumeric and underscores only)
        if !tenant_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(
                "Tenant name must contain only alphanumeric characters and underscores".into(),
            );
        }

        // Check if tenant already exists
        let mut tenants = self.tenants.lock().await;
        if tenants.contains_key(tenant_name) {
            return Err(format!("Tenant '{}' already exists", tenant_name).into());
        }

        // Create fake response
        let response = CreateTenantResponse {
            name: tenant_name.to_string(),
            status: "created".to_string(),
        };

        // Store the tenant
        tenants.insert(tenant_name.to_string(), response.clone());

        log::info!("Tenant '{}' created successfully (fake)", tenant_name);

        Ok(response)
    }

    pub async fn list_tenants(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let tenants = self.tenants.lock().await;
        let tenant_names: Vec<String> = tenants.keys().cloned().collect();
        Ok(tenant_names)
    }
}
