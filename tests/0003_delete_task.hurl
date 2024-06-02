# delete task

## create tasks

POST http://0.0.0.0:3000/create_task?select=id,typ
Prefer: return=representation
{
    "typ": "type_a"
}

HTTP 201

POST http://0.0.0.0:3000/create_task?select=id,typ
Prefer: return=representation
{
    "typ": "type_b"
}

HTTP 201

POST http://0.0.0.0:3000/create_task?select=id,typ
Prefer: return=representation
{
    "typ": "type_c"
}

HTTP 201

## change states
### running
POST http://0.0.0.0:3000/rpc/dequeue
Content-Profile: worker
Content-Type: application/json
{
    "timeout": 59
}

HTTP 200

[Captures]
running_task_id: jsonpath "$[0].id"


### completed
POST http://0.0.0.0:3000/rpc/dequeue
Content-Profile: worker
Content-Type: application/json
{
    "timeout": 59
}

HTTP 200

[Captures]
completed_task_id: jsonpath "$[0].id"

POST http://0.0.0.0:3000/rpc/complete_task
Content-Profile: worker
Content-Type: application/json
{
    "id": "{{completed_task_id}}"
}

HTTP 200

### queued

GET http://0.0.0.0:3000/task?state=eq.queued&limit=1&select=id

HTTP 200

[Captures]
queued_task_id: jsonpath "$[0].id"

## delete
POST http://0.0.0.0:3000/rpc/delete_task?select=id,state,updated_at
Prefer: return=representation
{
    "id": "{{queued_task_id}}"
}

HTTP 200

[Asserts]
jsonpath "$.id" == {{queued_task_id}}

POST http://0.0.0.0:3000/rpc/delete_task?select=id,state,updated_at
Prefer: return=representation
{
    "id": "{{running_task_id}}"
}

HTTP 200

[Asserts]
jsonpath "$.id" == null

POST http://0.0.0.0:3000/rpc/delete_task?select=id,state,updated_at
Prefer: return=representation
{
    "id": "{{completed_task_id}}"
}

HTTP 200

[Asserts]
jsonpath "$.id" == null
