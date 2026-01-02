use crate::config::Server;
use crate::tenant::{CreateTenantRequest, TenantManager};
use actix_web::{HttpRequest, HttpResponse, Result, web};
use std::sync::Arc;

pub async fn create_tenant(
    req: HttpRequest,
    tenant_request: web::Json<CreateTenantRequest>,
    tenant_manager: web::Data<Arc<TenantManager>>,
    config: web::Data<Server>,
) -> Result<HttpResponse> {
    // Validate API key
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let provided_key = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Ok(HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing or invalid Authorization header. Expected 'Bearer <api_key>'"})));
        }
    };

    if provided_key != config.tenant_api_key() {
        log::warn!("Invalid API key provided for tenant creation");
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({"error": "Invalid API key"})));
    }

    // Create tenant
    match tenant_manager.create_tenant(&tenant_request.name).await {
        Ok(response) => {
            log::info!("Tenant '{}' created successfully", tenant_request.name);
            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => {
            log::error!("Failed to create tenant '{}': {:?}", tenant_request.name, e);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})))
        }
    }
}

pub async fn list_tenants(
    req: HttpRequest,
    tenant_manager: web::Data<Arc<TenantManager>>,
    config: web::Data<Server>,
) -> Result<HttpResponse> {
    // Validate API key
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let provided_key = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Ok(HttpResponse::Unauthorized()
                .json(serde_json::json!({"error": "Missing or invalid Authorization header. Expected 'Bearer <api_key>'"})));
        }
    };

    if provided_key != config.tenant_api_key() {
        log::warn!("Invalid API key provided for tenant listing");
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({"error": "Invalid API key"})));
    }

    // List tenants
    match tenant_manager.list_tenants().await {
        Ok(tenants) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "tenants": tenants,
            "count": tenants.len()
        }))),
        Err(e) => {
            log::error!("Failed to list tenants: {:?}", e);
            Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Failed to list tenants"})))
        }
    }
}

pub fn scope() -> actix_web::Scope {
    web::scope("/api/tenants")
        .route("", web::post().to(create_tenant))
        .route("", web::get().to(list_tenants))
}
