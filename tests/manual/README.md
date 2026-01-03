# Manual Tests

These tests are not run automatically because they require real Azure OpenAI API calls and will incur costs.

## Prerequisites

1. Azure OpenAI credentials configured in `.env`
2. `FAKE_AI=false` in `.env`
3. Valid `AZURE_OPENAI_EMBEDDING_DEPLOYMENT_NAME` configured

## Running Manual Tests

### RAG (Retrieval-Augmented Generation) Tests

Tests the complete document storage and RAG query flow with entity-based filtering.

```bash
# Start the server
just run

# In another terminal, run the RAG tests
hurl --test --variable target=http://localhost:3000 --variable api_key=very-secure-one tests/manual/rag.hurl
```

## What These Tests Do

### rag.hurl
- Creates documents for multiple entities with real embeddings
- Stores them in the vector database
- Queries the chat endpoint with entity-specific context
- Verifies that entities only see their own documents
- Tests RAG responses contain relevant information from stored documents

**Note:** These tests will create real embeddings using Azure OpenAI and **will incur costs**.
