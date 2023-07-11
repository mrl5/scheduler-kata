# scheduler kata

## Motivation

* practice Rust + REST API + OAS
* practice PostgreSQL
* practice benchmarking


## Requirements

Expose an API that can:
* Create a task of a specific type and execution time, returning the task's ID
* Show a list of tasks, filterable by their state (whatever states you define) and/or their task type
* Show a task based on its ID
* Delete a task based on its ID
* The tasks must be persisted into some external data store (your choice).
* Process each task only once and only at/after their specified execution time.
* Support running multiple instances of your code in parallel.


## HOWTO dev

Check [CONTRIBUTING.md](./CONTRIBUTING.md)


## Build

```console
docker compose build
```


## Run

:warning:
Before first run make sure to bootstrap database
:warning:

For more info check [CONTRIBUTING.md](./CONTRIBUTING.md)

```console
docker compose up
```

http://0.0.0.0:8000/docs


## Iterations log

* [v0](https://github.com/mrl5/scheduler-kata/tree/v0) quick and dirty
  implementation where main focus was for exploring [axum
  framework](https://docs.rs/axum/latest/axum/) and [rust
  multithreading](https://kerkour.com/multithreading-in-rust)

* [v1](https://github.com/mrl5/scheduler-kata/tree/v1) going down the
  PostgreSQL rabbit hole and some sneak-peaking at this codebases:
  * https://github.com/windmill-labs/windmill
  * https://github.com/svix/svix-webhooks

* [v2](https://github.com/mrl5/scheduler-kata) **(ONGOING ITERATION)** mostly
  reusing `v1`, incorporating [aide](https://github.com/tamasfe/aide) for OAS
