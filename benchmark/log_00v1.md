# Benchmark no. 00

Captured state for commit
[7adc5dacb8](https://github.com/mrl5/scheduler-kata/tree/7adc5dacb81bf42af19d5390f1127b255c397a55)


## Content
* [Main focus](#main-focus)
* [Description](#description)
* [OS](#os)
* [Hardware](#hardware)
* [Setup](#setup)
* [Predicate 0](#predicate-0)
* [Predicate 1](#predicate-1)
* [Predicate 2](#predicate-2)
* [Predicate 3](#predicate-3)
* [Predicate 4](#predicate-4)
* [Additional notes](#additional-notes)


## Main focus
Database


## Description
Analyze query performance when both REST API, SCHEDULER and WORKERS are running


### Refs
* https://www.crunchydata.com/developers/playground/query-performance-analytics


## OS hosting just DB
```console
grep PRETTY_NAME /etc/os-release
```
```
PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
```
```console
uname -a
```
```
Linux DESKTOP-U9VGCHS 5.15.90.1-microsoft-standard-WSL2 #1 SMP Fri Jan 27 02:56:13 UTC 2023 x86_64 GNU/Linux
```


## Hardware hosting just DB
```console
lscpu
```
```
Architecture:            x86_64
  CPU op-mode(s):        32-bit, 64-bit
  Address sizes:         36 bits physical, 48 bits virtual
  Byte Order:            Little Endian
<REDACTED>
CPU(s):                  4
<REDACTED>
  Model name:            Intel(R) Core(TM) i5-2410M CPU @ 2.30GHz
<REDACTED>
```
```console
lsmem | grep -i 'total online'
```
```
Total online memory:       4G
```

Block device: Samsung SSD 850 PRO 256GB


## Setup

0. Make sure that DB is available from machine that's hosting REST API and
   WORKERS.

   You might need to modify `.env` file to specify `DB_HOST`. For me it's
   `DB_HOST=localhost` because I'm doing ssh tunnel to postgres server + `5432`
   port forwarding. Example `.ssh/config`:
```
Host vostro-tunnel
    Hostname DESKTOP-U9VGCHS
    Port 2222
    User kuba
    LocalForward 5432 0.0.0.0:5432
```

1. Record baseline
```console
ssh-add ~/.ssh/id_ed25519
rm -fv raw_stats_cpu.csv
date -Is -u >> raw_stats_cpu.csv
ssh vostro-tunnel docker stats --no-stream >> raw_stats_cpu.csv
```

2. Start app
```console
cp -v benchmark/.env .
docker compose up app-monolith worker
```

3. Start benchmark
```console
export PGPASSWORD=changeme1
bash ./benchmark/00_all-at-once/bench.sh
```

4. Stop containers once there is no active task in queue
```console
docker compose down app-monolith worker
```

5. Check predicates

## Predicate 0
### Basic Linux monitoring correlated with amount of data
1. Check `stats_cpu_*.csv` and `stats_db_*.csv`. Record `./benchmark/bench.sh`
   execution time
```
real	6m45.339s
user	0m20.399s
sys	0m6.936s
```

## Predicate 1
### Most time consuming queries
```
\x on
```
```sql
SELECT
  d.datname, round(s.total_exec_time::numeric, 2) AS total_exec_time, s.calls, s.rows,
  round(s.total_exec_time::numeric / calls, 2) AS avg_time,
  round((100 * s.total_exec_time / sum(s.total_exec_time::numeric) OVER ())::numeric, 2) AS percentage_cpu,
  substring(s.query, 1, 500) AS short_query
FROM pg_stat_statements s JOIN pg_database d ON (s.dbid = d.oid)
ORDER BY percentage_cpu DESC
LIMIT 5;
```
```
-[ RECORD 1 ]---+---------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 38675.91
calls           | 6000
rows            | 46819
avg_time        | 6.45
percentage_cpu  | 39.92
short_query     | SELECT                                                  +
                |             id,                                         +
                |             typ,                                        +
                |             state                                       +
                |         FROM task_state                                 +
                |          WHERE state = $1 ORDER BY id desc LIMIT $2
-[ RECORD 2 ]---+---------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 19597.11
calls           | 2000
rows            | 12000
avg_time        | 9.80
percentage_cpu  | 20.23
short_query     | SELECT                                                  +
                |             id,                                         +
                |             typ,                                        +
                |             state                                       +
                |         FROM task_state                                 +
                |          ORDER BY id desc LIMIT $1
-[ RECORD 3 ]---+---------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 15127.05
calls           | 3000
rows            | 29975
avg_time        | 5.04
percentage_cpu  | 15.61
short_query     | SELECT                                                  +
                |             id,                                         +
                |             typ,                                        +
                |             state                                       +
                |         FROM task_state                                 +
                |          WHERE typ = $1 ORDER BY id desc LIMIT $2
-[ RECORD 4 ]---+---------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 9495.05
calls           | 1000
rows            | 2000
avg_time        | 9.50
percentage_cpu  | 9.80
short_query     | SELECT                                                  +
                |             id,                                         +
                |             typ,                                        +
                |             state                                       +
                |         FROM task_state                                 +
                |          WHERE id < $1 ORDER BY id desc LIMIT $2
-[ RECORD 5 ]---+---------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 5308.63
calls           | 216
rows            | 13482
avg_time        | 24.58
percentage_cpu  | 5.48
short_query     | INSERT INTO queue (task_id, task_created_at, not_before)+
                |         SELECT id, created_at,                          +
                |             CASE                                        +
                |                 WHEN not_before IS NULL                 +
                |                     THEN created_at                     +
                |                 ELSE not_before                         +
                |             END                                         +
                |         FROM task_state WHERE state = $1::text          +
                |         ORDER BY id asc LIMIT $2                        +
                |         ON CONFLICT DO NOTHING
```


## Predicate 2
### Average Query Execution Time
```sql
SELECT (sum(total_exec_time) / sum(calls))::numeric(6,3) AS avg_execution_time
FROM pg_stat_statements;
```
```
-[ RECORD 1 ]------+------
avg_execution_time | 3.464
```


## Predicate 3
### Queries that write the most to shared_buffers
```sql
SELECT query, shared_blks_dirtied
FROM pg_stat_statements
WHERE shared_blks_dirtied > 0
ORDER BY 2 desc
LIMIT 5;
```
```
-[ RECORD 1 ]-------+----------------------------------------------------------------------------------------------
query               | INSERT INTO queue (task_id, task_created_at, not_before)                                     +
                    |         SELECT id, created_at,                                                               +
                    |             CASE                                                                             +
                    |                 WHEN not_before IS NULL                                                      +
                    |                     THEN created_at                                                          +
                    |                 ELSE not_before                                                              +
                    |             END                                                                              +
                    |         FROM task_state WHERE state = $1::text                                               +
                    |         ORDER BY id asc LIMIT $2                                                             +
                    |         ON CONFLICT DO NOTHING
shared_blks_dirtied | 141
-[ RECORD 2 ]-------+----------------------------------------------------------------------------------------------
query               | INSERT INTO task (                                                                           +
                    |             id,                                                                              +
                    |             typ,                                                                             +
                    |             not_before                                                                       +
                    |         )                                                                                    +
                    |         VALUES (                                                                             +
                    |             $1,                                                                              +
                    |             $2,                                                                              +
                    |             $3                                                                               +
                    |         )                                                                                    +
                    |         RETURNING id
shared_blks_dirtied | 104
-[ RECORD 3 ]-------+----------------------------------------------------------------------------------------------
query               | DELETE FROM tenant_default.queue
shared_blks_dirtied | 40
-[ RECORD 4 ]-------+----------------------------------------------------------------------------------------------
query               | WITH t AS (                                                                                  +
                    |             DELETE FROM queue                                                                +
                    |             WHERE (task_id, task_created_at) = ($1::uuid, $2::timestamptz)                   +
                    |             RETURNING task_id                                                                +
                    |         ) UPDATE task SET state = $3::text, inactive_since = now() FROM t                    +
                    |         WHERE id = t.task_id
shared_blks_dirtied | 36
-[ RECORD 5 ]-------+----------------------------------------------------------------------------------------------
query               | UPDATE tenant_default.task SET inactive_since = $1, state = $2 WHERE state = $3 OR state = $4
shared_blks_dirtied | 32
```


## Predicate 4
### Tables that might be needing an index
```sql
SELECT relname, seq_scan - idx_scan AS too_much_seq,
    CASE WHEN seq_scan - idx_scan > 0 THEN 'Missing Index?' ELSE 'OK' END,
    pg_relation_size(relid) AS rel_size, seq_scan, idx_scan
FROM pg_stat_user_tables
WHERE schemaname <> 'information_schema' AND schemaname NOT LIKE 'pg%'
ORDER BY too_much_seq DESC;
```
```
-[ RECORD 1 ]+-----------------
relname      | queue
too_much_seq | 1314764
case         | Missing Index?
rel_size     | 466944
seq_scan     | 1547954
idx_scan     | 233190
-[ RECORD 2 ]+-----------------
relname      | tenant
too_much_seq | 45
case         | Missing Index?
rel_size     | 8192
seq_scan     | 45
idx_scan     | 0
-[ RECORD 3 ]+-----------------
relname      | job
too_much_seq | 18
case         | Missing Index?
rel_size     | 8192
seq_scan     | 21
idx_scan     | 3
-[ RECORD 4 ]+-----------------
relname      | task
too_much_seq | 0
case         | OK
rel_size     | 0
seq_scan     | 0
idx_scan     | 0
-[ RECORD 5 ]+-----------------
relname      | _sqlx_migrations
too_much_seq | -2
case         | OK
rel_size     | 8192
seq_scan     | 2
idx_scan     | 4
-[ RECORD 6 ]+-----------------
relname      | job_run_details
too_much_seq | -153
case         | OK
rel_size     | 24576
seq_scan     | 19
idx_scan     | 172
-[ RECORD 7 ]+-----------------
relname      | task_y2023m07
too_much_seq | -206630
case         | OK
rel_size     | 483328
seq_scan     | 175606
idx_scan     | 382236
-[ RECORD 8 ]+-----------------
relname      | task_y2023m08
too_much_seq | -255686
case         | OK
rel_size     | 0
seq_scan     | 59934
idx_scan     | 315620
```


## Additional notes

### Warnings for SQLX logs
I see many warnings in worker logs:
```
WARN sqlx::query: summary="WITH t AS ( …" db.statement="\n\nWITH t AS (\n  SELECT\n    task_id as id,\n    task_created_at AS created_at\n  FROM\n    queue\n  WHERE\n    is_running = false\n    AND not_before <= now()\n  LIMIT\n    1 FOR\n  UPDATE\n    SKIP LOCKED\n)\nUPDATE\n  queue\nSET\n  is_running = true\nFROM\n  t\nWHERE\n  (task_id, task_created_at) = (t.id, t.created_at) RETURNING task_id,\n  task_created_at\n" rows_affected=0 rows_returned=1 elapsed=1.985186222s
```
sometimes elapsed is around **4 seconds !!!**

What's interesting no locks logged in DB although `log_lock_waits=on` and
`deadlock_timeout` is `1s`

### Collecting metrics made `bench.sh` execution longer by ~2minutes
Probably incorporating opentelemetry instead of periodic `ssh` (for gathering
CPU) would make some improvements
