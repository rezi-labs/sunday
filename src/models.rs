use crate::config::AzureOpenAIConfig;
use rig::client::{CompletionClient, EmbeddingsClient, ProviderClient};
use rig::embeddings::{EmbedError, EmbeddingModel, TextEmbedder};
use rig::providers::azure;
use rig::vector_store::VectorStoreIndex;
use rig_postgres::{PgSearchFilter, PgVectorDistanceFunction, PostgresVectorStore};
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
pub struct AiModels {
    client: azure::Client,
    pub deployment_name: String,
    pub embedding_deployment_name: String,
}

impl AiModels {
    /// Create AI models from Azure OpenAI configuration
    pub fn from_azure_config(config: &AzureOpenAIConfig) -> Result<Self, String> {
        // Set environment variables for Azure OpenAI
        unsafe {
            std::env::set_var("AZURE_API_KEY", &config.api_key);
            std::env::set_var("AZURE_API_VERSION", &config.api_version);
            std::env::set_var("AZURE_ENDPOINT", &config.endpoint);
        }

        // Create Azure OpenAI client from environment
        let client: azure::Client = ProviderClient::from_env();

        // Get embedding deployment name from environment
        let embedding_deployment_name = std::env::var("AZURE_OPENAI_EMBEDDING_DEPLOYMENT_NAME")
            .map_err(|_| {
                "AZURE_OPENAI_EMBEDDING_DEPLOYMENT_NAME must be set for embeddings".to_string()
            })?;

        Ok(AiModels {
            client,
            deployment_name: config.deployment_name.clone(),
            embedding_deployment_name,
        })
    }

    /// Create an agent for chat completion
    pub fn agent(&self, preamble: &str) -> rig::agent::Agent<azure::CompletionModel> {
        self.client
            .agent(&self.deployment_name)
            .preamble(preamble)
            .build()
    }

    /// Get the embedding model
    pub fn embedding_model(&self) -> azure::EmbeddingModel {
        self.client.embedding_model(&self.embedding_deployment_name)
    }

    /// Create a vector store for document retrieval
    pub fn vector_store(&self, pool: PgPool) -> PostgresVectorStore<azure::EmbeddingModel> {
        PostgresVectorStore::new(
            self.embedding_model(),
            pool,
            Some("documents".to_string()),
            PgVectorDistanceFunction::Cosine,
        )
    }

    /// Insert a document and link it to an entity
    pub async fn insert_document(
        &self,
        pool: &PgPool,
        content: &str,
        entity_id: Uuid,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        // Generate embedding
        let embedding_model = self.embedding_model();
        let embedding = embedding_model.embed_text(content).await?;

        // Insert document
        let document_id = Uuid::new_v4();
        let document_json = serde_json::to_value(&Document {
            content: content.to_string(),
        })?;
        let embedding_vec: Vec<f64> = embedding.vec;

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

        // Create filter for document IDs
        let filter = PgSearchFilter::member(
            "id".to_string(),
            document_ids
                .into_iter()
                .map(|id| serde_json::Value::String(id.to_string()))
                .collect(),
        );

        // Query vector store
        let vector_store = self.vector_store(pool.clone());
        let results: Vec<(f64, String, Document)> = vector_store
            .top_n(
                rig::vector_store::VectorSearchRequest::builder()
                    .query(query)
                    .samples(limit as u64)
                    .filter(filter)
                    .build()?,
            )
            .await?;

        // Extract content from results
        Ok(results.into_iter().map(|(_, _, doc)| doc.content).collect())
    }
}
