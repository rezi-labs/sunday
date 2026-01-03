# Test chat endpoint with entity_id

# Test 1: POST /api/chat without entity_id should fail
POST {{target}}/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "Hello"
    }
  ],
  "entity_id": ""
}

HTTP 400
[Asserts]
body contains "entity_id is required"


# Test 2: POST /api/chat with valid entity_id should succeed
POST {{target}}/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "Hello with entity"
    }
  ],
  "entity_id": "test-entity-123"
}

HTTP 200
[Asserts]
jsonpath "$.text" exists
jsonpath "$.error" not exists


# Test 3: POST /api/chat without auth should fail
POST {{target}}/api/chat
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "Hello"
    }
  ],
  "entity_id": "test-entity-123"
}

HTTP 403
[Asserts]
body contains "Authentication required"
