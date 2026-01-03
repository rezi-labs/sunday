-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create documents table with embeddings
-- Using 1536 dimensions for OpenAI text-embedding-3-small model
CREATE TABLE IF NOT EXISTS documents (
  id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
  document jsonb NOT NULL,
  embedded_text text NOT NULL,
  embedding vector(1536),
  created_at timestamptz DEFAULT now()
);

-- Create index on embeddings for faster similarity search
-- Using HNSW index with cosine distance (recommended for text embeddings)
CREATE INDEX IF NOT EXISTS document_embeddings_idx ON documents
USING hnsw(embedding vector_cosine_ops);

-- Create index on created_at for time-based queries
CREATE INDEX IF NOT EXISTS documents_created_at_idx ON documents(created_at DESC);
