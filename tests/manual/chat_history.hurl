# Test chat endpoint with message history

# Test 1: Single message (no history)
POST {{target}}/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "My name is Alice"
    }
  ],
  "entity_id": "test-entity-history"
}

HTTP 200
[Asserts]
jsonpath "$.text" exists
jsonpath "$.error" not exists


# Test 2: Follow-up message with history
POST {{target}}/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "My name is Alice"
    },
    {
      "role": "assistant",
      "text": "Nice to meet you, Alice! How can I help you today?"
    },
    {
      "role": "user",
      "text": "What is my name?"
    }
  ],
  "entity_id": "test-entity-history"
}

HTTP 200
[Asserts]
jsonpath "$.text" exists
jsonpath "$.text" contains "Alice"
jsonpath "$.error" not exists


# Test 3: Longer conversation history
POST {{target}}/api/chat
Authorization: Bearer {{api_key}}
Content-Type: application/json
{
  "messages": [
    {
      "role": "user",
      "text": "I like pizza"
    },
    {
      "role": "assistant",
      "text": "That's great! Pizza is delicious. What kind of pizza do you prefer?"
    },
    {
      "role": "user",
      "text": "I prefer pepperoni"
    },
    {
      "role": "assistant",
      "text": "Pepperoni is a classic choice! It's one of the most popular toppings."
    },
    {
      "role": "user",
      "text": "What food do I like and what topping did I mention?"
    }
  ],
  "entity_id": "test-entity-history"
}

HTTP 200
[Asserts]
jsonpath "$.text" exists
jsonpath "$.text" contains "pizza"
jsonpath "$.text" contains "pepperoni"
jsonpath "$.error" not exists
