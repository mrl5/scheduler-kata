# dequeue task

POST http://0.0.0.0:3000/rpc/dequeue
Content-Profile: worker
Content-Type: application/json
{
    "timeout": 59
}

HTTP 200

[Asserts]
jsonpath "$" isCollection
jsonpath "$" count == 1

[Captures]
task_id: jsonpath "$[0].id"

# complete task

POST http://0.0.0.0:3000/rpc/complete_task
Content-Profile: worker
Content-Type: application/json
{
    "id": "{{task_id}}"
}

HTTP 200

[Asserts]
jsonpath "$" isCollection
jsonpath "$" count == 1
jsonpath "$[0].id" == {{task_id}}
