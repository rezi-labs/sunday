# Test full RAG flow with entity-based documents
#
# IMPORTANT: This test requires real Azure OpenAI configuration with embeddings enabled.
# These tests are NOT run automatically - they must be run manually.
#
# To run these tests:
#   1. Ensure Azure OpenAI credentials are configured in .env
#   2. Set FAKE_AI=false in .env
#   3. Start the server: just run
#   4. Run tests: hurl --test --variable target=http://localhost:3000 --variable api_key=very-secure-one tests/manual/rag.hurl
#
# Note: These tests will create real embeddings using Azure OpenAI and will incur costs.

# Test 1: Create a document for entity "test-company"
POST {{target}}/api/documents
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "entity_id": "test-company",
  "content": "Acme Corporation was founded in 1920. We specialize in manufacturing anvils and explosives. Our headquarters is located in Phoenix, Arizona."
}

HTTP 201
[Asserts]
jsonpath "$.document_id" exists
jsonpath "$.entity_id" == "test-company"
jsonpath "$.message" contains "successfully"


# Test 2: Create another document for the same entity
POST {{target}}/api/documents
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "entity_id": "test-company",
  "content": "Our CEO is Jane Smith. She has been leading the company since 2015. Under her leadership, we expanded to 50 countries worldwide."
}

HTTP 201
[Asserts]
jsonpath "$.entity_id" == "test-company"


# Test 3: Create a document for a different entity
POST {{target}}/api/documents
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "entity_id": "other-company",
  "content": "TechCorp was founded in 2000. We focus on software development and cloud computing services."
}

HTTP 201
[Asserts]
jsonpath "$.entity_id" == "other-company"


# Test 4: Chat with first entity - should get context from its documents
POST {{target}}/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "entity_id": "test-company",
  "messages": [
    {
      "role": "user",
      "text": "When was the company founded?"
    }
  ]
}

HTTP 200
[Asserts]
jsonpath "$.text" exists
jsonpath "$.text" contains "1920"
jsonpath "$.error" not exists


# Test 5: Chat with first entity - should know about the CEO
POST {{target}}/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "entity_id": "test-company",
  "messages": [
    {
      "role": "user",
      "text": "Who is the CEO?"
    }
  ]
}

HTTP 200
[Asserts]
jsonpath "$.text" exists
jsonpath "$.text" contains "Jane"
jsonpath "$.error" not exists


# Test 6: Chat with second entity - should only see its own documents
POST {{target}}/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "entity_id": "other-company",
  "messages": [
    {
      "role": "user",
      "text": "When was the company founded?"
    }
  ]
}

HTTP 200
[Asserts]
jsonpath "$.text" exists
jsonpath "$.text" contains "2000"
jsonpath "$.text" not contains "1920"
jsonpath "$.error" not exists


# Test 7: Try to create document without entity_id
POST {{target}}/api/documents
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "entity_id": "",
  "content": "Some content"
}

HTTP 400
[Asserts]
jsonpath "$.error" contains "entity_id is required"


# Test 8: Try to create document with empty content
POST {{target}}/api/documents
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "entity_id": "test",
  "content": "   "
}

HTTP 400
[Asserts]
jsonpath "$.error" contains "content cannot be empty"


# Test 9: Try to create document without auth
POST {{target}}/api/documents
Content-Type: application/json
{
  "entity_id": "test",
  "content": "Some content"
}

HTTP 403
[Asserts]
body contains "Authentication required"
