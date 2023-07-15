GET http://0.0.0.0:8000/api/v1/task/list?state=pending

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection

GET http://0.0.0.0:8000/api/v1/task/list?state=deferred

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection

GET http://0.0.0.0:8000/api/v1/task/list?state=deleted

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection

GET http://0.0.0.0:8000/api/v1/task/list?state=processing

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection

GET http://0.0.0.0:8000/api/v1/task/list?state=failed

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection

GET http://0.0.0.0:8000/api/v1/task/list?state=done

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection

GET http://0.0.0.0:8000/api/v1/task/list?typ=type_a

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection

GET http://0.0.0.0:8000/api/v1/task/list?typ=type_b

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection

GET http://0.0.0.0:8000/api/v1/task/list?typ=type_c

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection

GET http://0.0.0.0:8000/api/v1/task/list?typ=type_c&state=deleted

HTTP 200

[Asserts]
jsonpath "$.tasks" isCollection
