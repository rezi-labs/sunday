use crate::api_key_middleware::AuthState;
use crate::tenant::{CreateTenantRequest, TenantManager};
use actix_web::{HttpResponse, Result, web};
use std::sync::Arc;

pub async fn create_tenant(
    auth_state: AuthState,
    tenant_request: web::Json<CreateTenantRequest>,
    tenant_manager: web::Data<Arc<TenantManager>>,
) -> Result<HttpResponse> {
    // Auth is enforced by AuthState extractor - if we reach here, we're authenticated
    log::debug!("Authenticated create tenant request: {:?}", auth_state);

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
    auth_state: AuthState,
    tenant_manager: web::Data<Arc<TenantManager>>,
) -> Result<HttpResponse> {
    // Auth is enforced by AuthState extractor - if we reach here, we're authenticated
    log::debug!("Authenticated list tenants request: {:?}", auth_state);

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
