# Test that authenticated endpoints require proper authentication

# NEGATIVE TESTS - Should fail without authentication

# Test 1: POST /api/chat without auth should fail
POST {{target}}/api/chat
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "Hello without auth"
    }
  ],
  "entity_id": "test-entity"
}

HTTP 403
[Asserts]
body contains "Authentication required"


# Test 2: POST /api/chat with invalid API key should fail
POST {{target}}/api/chat
Authorization: Bearer invalid-key-12345
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "Hello with bad auth"
    }
  ],
  "entity_id": "test-entity"
}

HTTP 403
[Asserts]
body contains "Authentication required"


# POSITIVE TESTS - Should succeed with valid authentication

# Test 3: POST /api/chat with valid API key should succeed
POST {{target}}/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "Hello with valid auth"
    }
  ],
  "entity_id": "test-entity"
}

HTTP 200
[Asserts]
jsonpath "$.text" exists
jsonpath "$.error" not exists
