# Test that authenticated endpoints require proper authentication

# NEGATIVE TESTS - Should fail without authentication

# Test 1: POST /api/tenants without auth should fail
POST {{target}}/api/tenants
Content-Type: application/json
{
  "name": "test-tenant-noauth"
}

HTTP 403
[Asserts]
body contains "Authentication required"


# Test 2: POST /api/tenants with invalid API key should fail
POST {{target}}/api/tenants
Authorization: Bearer invalid-key-12345
Content-Type: application/json
{
  "name": "test-tenant-badauth"
}

HTTP 403
[Asserts]
body contains "Authentication required"


# Test 3: GET /api/tenants without auth should fail
GET {{target}}/api/tenants

HTTP 403
[Asserts]
body contains "Authentication required"


# Test 4: GET /api/tenants with invalid API key should fail
GET {{target}}/api/tenants
Authorization: Bearer wrong-api-key

HTTP 403
[Asserts]
body contains "Authentication required"


# Test 5: POST /{tenant}/api/chat without auth should fail
POST {{target}}/acme/api/chat
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "Hello without auth"
    }
  ]
}

HTTP 403
[Asserts]
body contains "Authentication required"


# Test 6: POST /{tenant}/api/chat with invalid API key should fail
POST {{target}}/acme/api/chat
Authorization: Bearer invalid-key-67890
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "Hello with bad auth"
    }
  ]
}

HTTP 403
[Asserts]
body contains "Authentication required"


# POSITIVE TESTS - Should succeed with valid authentication

# Test 7: POST /api/tenants with valid API key should succeed
POST {{target}}/api/tenants
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "name": "test_tenant_auth"
}

HTTP 201
[Asserts]
jsonpath "$.name" == "test_tenant_auth"


# Test 8: GET /api/tenants with valid API key should succeed
GET {{target}}/api/tenants
Authorization: Bearer {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.tenants" isCollection
jsonpath "$.count" isInteger
jsonpath "$.count" >= 1


# Test 9: POST /{tenant}/api/chat with valid API key should succeed
POST {{target}}/acme/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "Hello with valid auth"
    }
  ]
}

HTTP 200
[Asserts]
jsonpath "$.text" exists
jsonpath "$.error" not exists
