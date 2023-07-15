## list tasks
GET http://0.0.0.0:8000/api/v1/task/list

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection
jsonpath "$.per_page" isInteger
