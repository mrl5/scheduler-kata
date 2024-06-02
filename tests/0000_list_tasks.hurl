# list tasks
GET http://0.0.0.0:3000/task

HTTP 200

[Asserts]
jsonpath "$" isCollection
