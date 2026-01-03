use actix_web::{HttpResponse, Result as AwResult, post, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateDocumentRequest {
    pub entity_id: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct CreateDocumentResponse {
    pub document_id: String,
    pub entity_id: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[post("/documents")]
pub async fn create_document(
    _auth_state: crate::api_key_middleware::AuthState,
    body: web::Json<CreateDocumentRequest>,
    pool: web::Data<PgPool>,
    ai_models: Option<web::Data<std::sync::Arc<crate::models::AiModels>>>,
) -> AwResult<HttpResponse> {
    // Validate entity_id
    if body.entity_id.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "entity_id is required".to_string(),
        }));
    }

    // Validate content
    if body.content.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "content cannot be empty".to_string(),
        }));
    }

    // Check if we have AI models for embeddings
    if ai_models.is_none() {
        return Ok(HttpResponse::ServiceUnavailable().json(ErrorResponse {
            error: "AI models not available for embedding generation".to_string(),
        }));
    }

    let ai_models = ai_models.unwrap();

    // Get or create entity
    let entity_id = match get_or_create_entity(&pool, &body.entity_id).await {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to get or create entity: {}", e);
            return Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to process entity: {}", e),
            }));
        }
    };

    // Generate embedding and insert document
    match ai_models
        .insert_document(&pool, &body.content, entity_id)
        .await
    {
        Ok(document_id) => {
            log::info!(
                "Document {} created and linked to entity {}",
                document_id,
                entity_id
            );
            Ok(HttpResponse::Created().json(CreateDocumentResponse {
                document_id: document_id.to_string(),
                entity_id: entity_id.to_string(),
                message: "Document created and embedded successfully".to_string(),
            }))
        }
        Err(e) => {
            log::error!("Failed to create document: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to create document: {}", e),
            }))
        }
    }
}

async fn get_or_create_entity(pool: &PgPool, entity_name: &str) -> Result<Uuid, sqlx::Error> {
    // Try to get existing entity
    let result: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM entities WHERE name = $1")
        .bind(entity_name)
        .fetch_optional(pool)
        .await?;

    if let Some((id,)) = result {
        return Ok(id);
    }

    // Create new entity
    let result: (Uuid,) = sqlx::query_as("INSERT INTO entities (name) VALUES ($1) RETURNING id")
        .bind(entity_name)
        .fetch_one(pool)
        .await?;

    log::info!("Created new entity: {} with id {}", entity_name, result.0);
    Ok(result.0)
}
