GET {{target}}/health

HTTP 200

[Asserts]
jsonpath "$.status" == "ok"
jsonpath "$.message" == "Service is healthy"
