GET http://localhost:8000/api/v1/task

HTTP 400

GET http://localhost:8000/api/v1/task?id=00a0a0aa-a0aa-00aa-0000-0000a000a000

HTTP 404


POST http://localhost:8000/api/v1/task
{
  "task_type": "type_c"
}

HTTP 202

[Asserts]
jsonpath "$.id" exists

[Captures]
new_c_task_id: jsonpath "$.id"


GET http://localhost:8000/api/v1/task?id={{new_c_task_id}}

HTTP 200

[Asserts]
jsonpath "$.id" == {{new_c_task_id}}
jsonpath "$.state" == null # todo: "created"
