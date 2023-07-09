POST http://0.0.0.0:8000/api/v1/task
{
  "task_type": "type_b",
  "not_before": "{{tomorrow}}"
}

HTTP 202

[Asserts]
jsonpath "$.id" exists
