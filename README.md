# scheduler kata

Grinding the api-queue-worker pattern for fun

## Motivation

* practice PostgreSQL
* practice PostgREST
* practice benchmarking
* practice Rust


## Requirements

Expose an API that can:
* Create a task of a specific type and execution time, returning the task's ID
* Show a list of tasks, filterable by their state (whatever states you define)
  and/or their task type
* Show a task based on its ID
* Delete a task based on its ID
* The tasks must be persisted into some external data store (your choice).
* Process each task only once and only at/after their specified execution time.
* Support running multiple instances of your code in parallel.


## Additional materials
* [Queues in Postgres | Postgres.FM 042 | #PostgreSQL #Postgres
  podcast](https://www.youtube.com/watch?v=mW5z5NYpGeA) on YouTube
* [PGMQ](https://github.com/tembo-io/pgmq) A lightweight message queue. Like
  AWS SQS and RSMQ but on Postgres
* [River](https://github.com/riverqueue/river) Atomic, transaction-safe, robust
  job queueing for Go applications. Backed by PostgreSQL and built to scale


## Iterations log

* [v0](https://github.com/mrl5/scheduler-kata/tree/v0) quick and dirty
  implementation where main focus was for exploring [axum
  framework](https://docs.rs/axum/latest/axum/) and [rust
  multithreading](https://kerkour.com/multithreading-in-rust)

* [v1](https://github.com/mrl5/scheduler-kata/tree/v1) going down the
  PostgreSQL rabbit hole and some sneak-peaking at this codebases:
  * https://github.com/windmill-labs/windmill
  * https://github.com/svix/svix-webhooks

* [v2](https://github.com/mrl5/scheduler-kata/tree/v2) mostly reusing `v1`,
  incorporating [aide](https://github.com/tamasfe/aide) for OAS, bundling
  scheduler, queue and worker

* [v3](https://github.com/mrl5/scheduler-kata/tree/v3) telemetry, benchmarking and
  performance improvements

* [v4](https://github.com/mrl5/scheduler-kata) new approach inspired by
  [PGMQ](https://github.com/tembo-io/pgmq) but also other additional materials
