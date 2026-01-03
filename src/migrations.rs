use sqlx::postgres::PgPoolOptions;

pub async fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Running database migrations...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Enable pgvector extension
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(&pool)
        .await?;

    // Create documents table with embeddings
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS documents (
          id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
          document jsonb NOT NULL,
          embedded_text text NOT NULL,
          embedding vector(1536),
          created_at timestamptz DEFAULT now()
        )
    "#,
    )
    .execute(&pool)
    .await?;

    // Create index on embeddings for faster similarity search
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS document_embeddings_idx ON documents
        USING hnsw(embedding vector_cosine_ops)
    "#,
    )
    .execute(&pool)
    .await?;

    // Create index on created_at for time-based queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS documents_created_at_idx ON documents(created_at DESC)",
    )
    .execute(&pool)
    .await?;

    // Create entities table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS entities (
          id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
          name text NOT NULL UNIQUE,
          created_at timestamptz DEFAULT now()
        )
    "#,
    )
    .execute(&pool)
    .await?;

    // Create entity_documents junction table (many-to-many)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS entity_documents (
          entity_id uuid NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
          document_id uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
          created_at timestamptz DEFAULT now(),
          PRIMARY KEY (entity_id, document_id)
        )
    "#,
    )
    .execute(&pool)
    .await?;

    // Create indexes for entity_documents junction table
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS entity_documents_entity_idx ON entity_documents(entity_id)",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS entity_documents_document_idx ON entity_documents(document_id)",
    )
    .execute(&pool)
    .await?;

    log::info!("Database migrations completed successfully");

    pool.close().await;

    Ok(())
}
