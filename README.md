# scheduler kata

## Requirements

Expose an API that can:
* Create a task of a specific type and execution time, returning the task's ID
* Show a list of tasks, filterable by their state (whatever states you define) and/or their task type
* Show a task based on its ID
* Delete a task based on its ID
* The tasks must be persisted into some external data store (your choice).
* Process each task only once and only at/after their specified execution time.
* Support running multiple instances of your code in parallel.

## Personal agenda

* practice Rust + REST API + OAS
* practice PostgreSQL

## HOWTO dev

Check [CONTRIBUTING.md](./CONTRIBUTING.md)

## Build

```console
docker compose build
```

## Run

:warning:
Before first run make sure to bootstrap database. For more info check
[CONTRIBUTING.md](./CONTRIBUTING.md)
:warning:

```console
docker compose up
```

http://localhost:8000/docs
