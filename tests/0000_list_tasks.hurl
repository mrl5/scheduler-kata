# list tasks
GET http://0.0.0.0:3000/task?limit=10

HTTP 200

[Asserts]
jsonpath "$" isCollection
