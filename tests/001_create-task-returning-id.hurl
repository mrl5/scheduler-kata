POST http://localhost:8000/api/v1/task/create
{
  "task_type": "type_a"
}

HTTP 202

[Asserts]
jsonpath "$.id" exists

POST http://localhost:8000/api/v1/task/create
{
  "task_type": "type_b"
}

HTTP 202

[Asserts]
jsonpath "$.id" exists

POST http://localhost:8000/api/v1/task/create
{
  "task_type": "type_c"
}

HTTP 202

[Asserts]
jsonpath "$.id" exists

POST http://localhost:8000/api/v1/task/create
{
  "task_type": "type_d"
}

HTTP 400
