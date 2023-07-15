GET http://0.0.0.0:8000/api/v1/task/list?per_page=2

HTTP 200

[Asserts]
jsonpath "$.tasks[2]" not exists
jsonpath "$.per_page" == 2
jsonpath "$.anchor" exists

[Captures]
anchor: jsonpath "$.anchor"

GET http://0.0.0.0:8000/api/v1/task/list?per_page=2&anchor={{anchor}}

HTTP 200

[Asserts]
jsonpath "$.tasks[2]" not exists
jsonpath "$.per_page" == 2
