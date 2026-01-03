use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::{HttpResponse, Result as AwResult, post, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateDocumentRequest {
    pub entity_id: String,
    pub content: String,
}

#[derive(Debug, MultipartForm)]
pub struct UploadDocumentForm {
    #[multipart(rename = "file")]
    pub file: TempFile,
    #[multipart(rename = "entity_id")]
    pub entity_id: actix_multipart::form::text::Text<String>,
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

#[post("/documents/upload")]
pub async fn upload_document(
    _auth_state: crate::api_key_middleware::AuthState,
    MultipartForm(form): MultipartForm<UploadDocumentForm>,
    pool: web::Data<PgPool>,
    ai_models: Option<web::Data<std::sync::Arc<crate::models::AiModels>>>,
) -> AwResult<HttpResponse> {
    // Validate entity_id
    let entity_id = form.entity_id.0.trim();
    if entity_id.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "entity_id is required".to_string(),
        }));
    }

    // Check if we have AI models for embeddings
    if ai_models.is_none() {
        return Ok(HttpResponse::ServiceUnavailable().json(ErrorResponse {
            error: "AI models not available for embedding generation".to_string(),
        }));
    }

    let ai_models = ai_models.unwrap();

    // Get file information
    let file_name = form
        .file
        .file_name
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let file_path = form.file.file.path();

    log::info!("Processing uploaded file: {}", file_name);

    // Extract text based on file type
    let content = if file_name.to_lowercase().ends_with(".pdf") {
        match extract_pdf_text(file_path) {
            Ok(text) => text,
            Err(e) => {
                log::error!("Failed to extract text from PDF: {}", e);
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    error: format!("Failed to extract text from PDF: {}", e),
                }));
            }
        }
    } else if file_name.to_lowercase().ends_with(".txt") {
        match std::fs::read_to_string(file_path) {
            Ok(text) => text,
            Err(e) => {
                log::error!("Failed to read text file: {}", e);
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    error: format!("Failed to read text file: {}", e),
                }));
            }
        }
    } else {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Unsupported file type. Only .pdf and .txt files are supported".to_string(),
        }));
    };

    // Validate content
    if content.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Extracted content is empty".to_string(),
        }));
    }

    log::info!("Extracted {} characters from {}", content.len(), file_name);

    // Split content into chunks for better embedding quality
    let chunks = chunk_text(&content, 1000, 200);

    if chunks.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Failed to create chunks from content".to_string(),
        }));
    }

    log::info!("Split content into {} chunks", chunks.len());

    // Get or create entity
    let entity_uuid = match get_or_create_entity(&pool, entity_id).await {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to get or create entity: {}", e);
            return Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to process entity: {}", e),
            }));
        }
    };

    // Generate embeddings and insert document chunks
    match ai_models
        .insert_document_chunks(&pool, chunks, entity_uuid)
        .await
    {
        Ok(document_ids) => {
            log::info!(
                "Created {} document chunks from file {} and linked to entity {}",
                document_ids.len(),
                file_name,
                entity_uuid
            );
            Ok(HttpResponse::Created().json(CreateDocumentResponse {
                document_id: document_ids
                    .first()
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
                entity_id: entity_uuid.to_string(),
                message: format!(
                    "Document from '{}' split into {} chunks and embedded successfully",
                    file_name,
                    document_ids.len()
                ),
            }))
        }
        Err(e) => {
            log::error!("Failed to create document chunks: {}", e);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to create document: {}", e),
            }))
        }
    }
}

fn extract_pdf_text(file_path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(file_path).map_err(|e| format!("Failed to read PDF file: {}", e))?;

    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| format!("Failed to extract text from PDF: {}", e))?;

    if text.trim().is_empty() {
        Err("PDF appears to be empty or contains only images".to_string())
    } else {
        Ok(text)
    }
}

/// Split text into chunks with overlap for better context preservation
/// Default chunk size: 1000 characters, overlap: 200 characters
fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    let text_len = text.len();

    while start < text_len {
        let end = std::cmp::min(start + chunk_size, text_len);

        // Try to break at a sentence boundary (. ! ?) followed by whitespace
        let chunk_end = if end < text_len {
            let search_start = std::cmp::max(start, end.saturating_sub(200));
            text[search_start..end]
                .rfind(['.', '!', '?'])
                .map(|pos| {
                    let abs_pos = search_start + pos + 1;
                    // Skip whitespace after punctuation
                    text[abs_pos..end]
                        .find(|c: char| !c.is_whitespace())
                        .map(|ws_offset| abs_pos + ws_offset)
                        .unwrap_or(end)
                })
                .unwrap_or(end)
        } else {
            end
        };

        chunks.push(text[start..chunk_end].trim().to_string());

        // Move start forward, accounting for overlap
        if chunk_end >= text_len {
            break;
        }
        start = chunk_end.saturating_sub(overlap);
    }

    chunks
}
