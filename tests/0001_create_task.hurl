# create task

POST http://0.0.0.0:3000/create_task?select=id
Prefer: return=representation
{
    "id": "00000000-0000-0000-0000-000000000000",
    "typ": "type_a"
}

HTTP 201

[Asserts]
jsonpath "$" count == 1
jsonpath "$[0].id" exists
jsonpath "$[0].id" != "00000000-0000-0000-0000-000000000000"
