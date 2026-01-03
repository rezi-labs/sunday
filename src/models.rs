use crate::config::AzureOpenAIConfig;
use pgvector::Vector;
use rig::client::{CompletionClient, EmbeddingsClient};
use rig::embeddings::{EmbedError, EmbeddingModel, TextEmbedder};
use rig::providers::azure;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Document {
    pub content: String,
}

impl rig::Embed for Document {
    fn embed(&self, embedder: &mut TextEmbedder) -> Result<(), EmbedError> {
        embedder.embed(self.content.clone());
        Ok(())
    }
}

/// Container for Azure OpenAI client and deployment names
#[derive(Clone)]
pub struct AiModels {
    chat_client: azure::Client,
    embedding_client: azure::Client,
    pub deployment_name: String,
    pub embedding_deployment_name: String,
}

impl AiModels {
    /// Create AI models from Azure OpenAI configuration
    pub fn from_azure_config(config: &AzureOpenAIConfig) -> Result<Self, String> {
        // Read Azure configuration from environment
        let api_key =
            std::env::var("AZURE_API_KEY").map_err(|_| "AZURE_API_KEY must be set".to_string())?;

        let chat_api_version = std::env::var("AZURE_CHAT_API_VERSION")
            .map_err(|_| "AZURE_CHAT_API_VERSION must be set".to_string())?;

        let embedding_api_version = std::env::var("AZURE_EMBEDDING_API_VERSION")
            .map_err(|_| "AZURE_EMBEDDING_API_VERSION must be set".to_string())?;

        let azure_endpoint = std::env::var("AZURE_ENDPOINT")
            .map_err(|_| "AZURE_ENDPOINT must be set".to_string())?;

        log::info!("Azure OpenAI endpoint: {}", azure_endpoint);
        log::info!("Chat API version: {}", chat_api_version);
        log::info!("Embedding API version: {}", embedding_api_version);
        log::info!("Chat deployment name: {}", config.deployment_name);

        // Create Azure OpenAI client for chat completions
        // Use base_url for the endpoint, set azure_endpoint to empty string
        let chat_client: azure::Client = azure::Client::builder()
            .base_url(azure_endpoint.clone())
            .api_key(azure::AzureOpenAIAuth::ApiKey(api_key.clone()))
            .azure_endpoint("".to_string())
            .api_version(&chat_api_version)
            .build()
            .map_err(|e| format!("Failed to build Azure chat client: {}", e))?;

        log::info!("Azure chat client built successfully");

        // Get embedding deployment name from environment
        let embedding_deployment_name = std::env::var("AZURE_OPENAI_EMBEDDING_DEPLOYMENT_NAME")
            .map_err(|_| {
                "AZURE_OPENAI_EMBEDDING_DEPLOYMENT_NAME must be set for embeddings".to_string()
            })?;

        log::info!("Embedding deployment name: {}", embedding_deployment_name);

        // Create separate Azure OpenAI client for embeddings with different API version
        // Use base_url for the endpoint, set azure_endpoint to empty string
        let embedding_client: azure::Client = azure::Client::builder()
            .base_url(azure_endpoint.clone())
            .api_key(azure::AzureOpenAIAuth::ApiKey(api_key))
            .azure_endpoint("".to_string())
            .api_version(&embedding_api_version)
            .build()
            .map_err(|e| format!("Failed to build Azure embedding client: {}", e))?;

        log::info!("Azure embedding client built successfully");

        Ok(AiModels {
            chat_client,
            embedding_client,
            deployment_name: config.deployment_name.clone(),
            embedding_deployment_name,
        })
    }

    /// Create an agent for chat completion
    pub fn agent(&self, preamble: &str) -> rig::agent::Agent<azure::CompletionModel> {
        self.chat_client
            .agent(&self.deployment_name)
            .preamble(preamble)
            .build()
    }

    /// Get the embedding model
    pub fn embedding_model(&self) -> azure::EmbeddingModel {
        log::debug!(
            "Creating embedding model with deployment: {}",
            self.embedding_deployment_name
        );
        self.embedding_client
            .embedding_model(&self.embedding_deployment_name)
    }

    /// Insert a document and link it to an entity
    pub async fn insert_document(
        &self,
        pool: &PgPool,
        content: &str,
        entity_id: Uuid,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        // Generate embedding
        log::debug!(
            "Generating embedding for content (length: {})",
            content.len()
        );

        // Log the expected URL format for Azure OpenAI embeddings
        let azure_endpoint = std::env::var("AZURE_ENDPOINT").unwrap_or_default();
        let embedding_api_version =
            std::env::var("AZURE_EMBEDDING_API_VERSION").unwrap_or_default();
        let expected_url = format!(
            "{}/openai/deployments/{}/embeddings?api-version={}",
            azure_endpoint, self.embedding_deployment_name, embedding_api_version
        );
        log::info!("Expected Azure OpenAI embedding URL: {}", expected_url);

        let embedding_model = self.embedding_model();
        log::debug!("Calling Azure OpenAI embedding API...");
        let embedding = embedding_model.embed_text(content).await?;
        log::debug!(
            "Successfully received embedding with {} dimensions",
            embedding.vec.len()
        );

        // Insert document
        let document_id = Uuid::new_v4();
        let document_json = serde_json::to_value(&Document {
            content: content.to_string(),
        })?;
        // Convert f64 to f32 for pgvector
        let embedding_vec: Vec<f32> = embedding.vec.iter().map(|&x| x as f32).collect();
        let embedding_vec = Vector::from(embedding_vec);

        sqlx::query(
            "INSERT INTO documents (id, document, embedded_text, embedding) VALUES ($1, $2, $3, $4)",
        )
        .bind(document_id)
        .bind(&document_json)
        .bind(content)
        .bind(&embedding_vec)
        .execute(pool)
        .await?;

        // Link document to entity
        sqlx::query("INSERT INTO entity_documents (entity_id, document_id) VALUES ($1, $2)")
            .bind(entity_id)
            .bind(document_id)
            .execute(pool)
            .await?;

        Ok(document_id)
    }

    /// Insert multiple document chunks and link them to an entity
    /// Returns the document IDs of all inserted chunks
    pub async fn insert_document_chunks(
        &self,
        pool: &PgPool,
        chunks: Vec<String>,
        entity_id: Uuid,
    ) -> Result<Vec<Uuid>, Box<dyn std::error::Error>> {
        let mut document_ids = Vec::new();

        for chunk in chunks {
            if chunk.trim().is_empty() {
                continue;
            }

            let document_id = self.insert_document(pool, &chunk, entity_id).await?;
            document_ids.push(document_id);
        }

        Ok(document_ids)
    }

    /// Query documents for an entity using vector similarity
    pub async fn query_entity_documents(
        &self,
        pool: &PgPool,
        entity_id: Uuid,
        query: &str,
        limit: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Get document IDs for this entity
        let document_ids: Vec<Uuid> =
            sqlx::query_as("SELECT document_id FROM entity_documents WHERE entity_id = $1")
                .bind(entity_id)
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(|(id,)| id)
                .collect();

        if document_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Generate embedding for the query
        let embedding_model = self.embedding_model();
        let query_embedding = embedding_model.embed_text(query).await?;
        // Convert f64 to f32 for pgvector
        let query_vec: Vec<f32> = query_embedding.vec.iter().map(|&x| x as f32).collect();
        let query_vec = Vector::from(query_vec);

        // Query using direct SQL with pgvector L2 distance
        // Using ANY($2) to filter by document IDs array
        let results: Vec<(String,)> = sqlx::query_as(
            "SELECT embedded_text
             FROM documents
             WHERE id = ANY($1)
             ORDER BY embedding <=> $2
             LIMIT $3",
        )
        .bind(&document_ids)
        .bind(&query_vec)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

        // Extract content from results
        Ok(results.into_iter().map(|(content,)| content).collect())
    }
}
