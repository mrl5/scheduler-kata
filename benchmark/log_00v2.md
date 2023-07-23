# Benchmark no. 00

Captured state for commit
[a624fdef88](https://github.com/mrl5/scheduler-kata/tree/a624fdef88fcc9942fef3874db2cfa837793876d)


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
-[ RECORD 1 ]---+-----------------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 38875.05
calls           | 6000
rows            | 46630
avg_time        | 6.48
percentage_cpu  | 32.80
short_query     | SELECT                                                          +
                |             id,                                                 +
                |             typ,                                                +
                |             state                                               +
                |         FROM task_state                                         +
                |          WHERE state = $1 ORDER BY id desc LIMIT $2
-[ RECORD 2 ]---+-----------------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 29575.73
calls           | 3000
rows            | 29975
avg_time        | 9.86
percentage_cpu  | 24.95
short_query     | SELECT                                                          +
                |             id,                                                 +
                |             typ,                                                +
                |             state                                               +
                |         FROM task_state                                         +
                |          WHERE typ = $1 ORDER BY id desc LIMIT $2
-[ RECORD 3 ]---+-----------------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 21117.41
calls           | 2000
rows            | 12000
avg_time        | 10.56
percentage_cpu  | 17.81
short_query     | SELECT                                                          +
                |             id,                                                 +
                |             typ,                                                +
                |             state                                               +
                |         FROM task_state                                         +
                |          ORDER BY id desc LIMIT $1
-[ RECORD 4 ]---+-----------------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 10249.76
calls           | 1000
rows            | 2000
avg_time        | 10.25
percentage_cpu  | 8.65
short_query     | SELECT                                                          +
                |             id,                                                 +
                |             typ,                                                +
                |             state                                               +
                |         FROM task_state                                         +
                |          WHERE id < $1 ORDER BY id desc LIMIT $2
-[ RECORD 5 ]---+-----------------------------------------------------------------
datname         | scheduler-kata
total_exec_time | 7062.29
calls           | 1000
rows            | 9975
avg_time        | 7.06
percentage_cpu  | 5.96
short_query     | SELECT                                                          +
                |             id,                                                 +
                |             typ,                                                +
                |             state                                               +
                |         FROM task_state                                         +
                |          WHERE typ = $1 AND state = $2 ORDER BY id desc LIMIT $3

```


## Predicate 2
### Average Query Execution Time
```sql
SELECT (sum(total_exec_time) / sum(calls))::numeric(6,3) AS avg_execution_time
FROM pg_stat_statements;
```
```
-[ RECORD 1 ]------+------
avg_execution_time | 4.243
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
-[ RECORD 1 ]-------+--------------------------------------------------------------------------------
query               | WITH q AS (                                                                    +
                    |     DELETE FROM task_bucket USING (                                            +
                    |         SELECT id FROM task_bucket LIMIT $1 FOR UPDATE SKIP LOCKED             +
                    |     ) t                                                                        +
                    |     WHERE t.id = task_bucket.id                                                +
                    |     RETURNING task_bucket.id, task_bucket.created_at, task_bucket.not_before   +
                    | ) INSERT INTO queue (task_id, task_created_at, not_before)                     +
                    | SELECT id, created_at,                                                         +
                    |     CASE                                                                       +
                    |         WHEN not_before IS NULL                                                +
                    |             THEN created_at                                                    +
                    |         ELSE not_before                                                        +
                    |     END                                                                        +
                    | from q ON CONFLICT DO NOTHING
shared_blks_dirtied | 281
-[ RECORD 2 ]-------+--------------------------------------------------------------------------------
query               | WITH t AS (                                                                    +
                    |     INSERT INTO task (                                                         +
                    |         id,                                                                    +
                    |         typ,                                                                   +
                    |         not_before                                                             +
                    |     )                                                                          +
                    |     VALUES (                                                                   +
                    |         $1,                                                                    +
                    |         $2,                                                                    +
                    |         $3                                                                     +
                    |     ) RETURNING created_at, not_before, id                                     +
                    | ) INSERT INTO task_bucket SELECT created_at, not_before, id from t RETURNING id
shared_blks_dirtied | 171
-[ RECORD 3 ]-------+--------------------------------------------------------------------------------
query               | INSERT INTO tenant_default.task_bucket SELECT created_at, not_before, id       +
                    |             FROM tenant_default.task WHERE tenant_default.task.state IS null
shared_blks_dirtied | 70
-[ RECORD 4 ]-------+--------------------------------------------------------------------------------
query               | WITH t AS (                                                                    +
                    |             DELETE FROM queue                                                  +
                    |             WHERE (task_id, task_created_at) = ($1::uuid, $2::timestamptz)     +
                    |             RETURNING task_id                                                  +
                    |         ) UPDATE task SET state = $3::text, inactive_since = now() FROM t      +
                    |         WHERE id = t.task_id
shared_blks_dirtied | 64
-[ RECORD 5 ]-------+--------------------------------------------------------------------------------
query               | DELETE FROM tenant_default.queue
shared_blks_dirtied | 17
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
relname      | task_bucket
too_much_seq | 2401
case         | Missing Index?
rel_size     | 745472
seq_scan     | 2401
idx_scan     | 0
-[ RECORD 2 ]+-----------------
relname      | tenant
too_much_seq | 1
case         | Missing Index?
rel_size     | 8192
seq_scan     | 1
idx_scan     | 0
-[ RECORD 3 ]+-----------------
relname      | job_run_details
too_much_seq | 1
case         | Missing Index?
rel_size     | 0
seq_scan     | 1
idx_scan     | 0
-[ RECORD 4 ]+-----------------
relname      | job
too_much_seq | 1
case         | Missing Index?
rel_size     | 8192
seq_scan     | 5
idx_scan     | 4
-[ RECORD 5 ]+-----------------
relname      | task
too_much_seq | 0
case         | OK
rel_size     | 0
seq_scan     | 0
idx_scan     | 0
-[ RECORD 6 ]+-----------------
relname      | _sqlx_migrations
too_much_seq | -4
case         | OK
rel_size     | 8192
seq_scan     | 2
idx_scan     | 6
-[ RECORD 7 ]+-----------------
relname      | task_y2023m08
too_much_seq | -25706
case         | OK
rel_size     | 0
seq_scan     | 6229
idx_scan     | 31935
-[ RECORD 8 ]+-----------------
relname      | task_y2023m07
too_much_seq | -45010
case         | OK
rel_size     | 507904
seq_scan     | 5626
idx_scan     | 50636
-[ RECORD 9 ]+-----------------
relname      | queue
too_much_seq | -2295464
case         | OK
rel_size     | 1605632
seq_scan     | 173534
idx_scan     | 2468998
```
