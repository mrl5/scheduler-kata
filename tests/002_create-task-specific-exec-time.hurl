POST http://localhost:8000/api/v1/task/create
{
  "task_type": "type_b",
  "not_before": "{{tomorrow}}"
}

HTTP 202

[Asserts]
jsonpath "$.id" exists
