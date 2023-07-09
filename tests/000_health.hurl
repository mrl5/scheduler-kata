# basic health check
GET http://localhost:8000/health

HTTP 200

[Asserts]
jsonpath "$.status" == "Healthy"
