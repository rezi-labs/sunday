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

    log::info!("Database migrations completed successfully");

    pool.close().await;

    Ok(())
}
