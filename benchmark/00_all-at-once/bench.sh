#!/bin/bash

DB_STATS_FILE="raw_stats_db.csv"
CPU_STATS_FILE="raw_stats_cpu.csv"

main() {
    collect_db_metrics
    collect_cpu_metrics
    reset_pg_stat_statements

    for i in $(seq 5); do
        for x in $(seq 20); do
            do_http_traffic
            collect_db_metrics &
            collect_cpu_metrics &
        done

        rotate_tasks
        collect_db_metrics

        for x in $(seq 20); do
            do_http_traffic
            collect_db_metrics &
            collect_cpu_metrics &
        done
    done
}

reset_pg_stat_statements() {
    psql -h localhost -U postgres -d scheduler-kata \
        -c 'SELECT pg_stat_statements_reset()'
}

do_http_traffic() {
    just test-api &
    just test-api &
    just test-api &
    just test-api &
    just test-api &
    wait
}

collect_db_metrics() {
    psql -h localhost -U postgres -d scheduler-kata \
    --csv -c "WITH q as (
  SELECT COUNT(1) AS queue_length FROM tenant_default.queue
) SELECT
  now() as timestamp,
  COUNT(1) AS task_count,
  (SELECT * FROM q),
  (
    SELECT COUNT(1) FROM pg_stat_activity
    WHERE datname = 'scheduler-kata' AND application_name = 'worker'
  ) as worker_conn_count,
  (
    SELECT COUNT(1) FROM pg_stat_activity
    WHERE datname = 'scheduler-kata' AND application_name = 'app-monolith'
  ) as api_conn_count
FROM tenant_default.task" \
    >> "$DB_STATS_FILE"
}

collect_cpu_metrics() {
    date -Is -u >> "$CPU_STATS_FILE"
    ssh vostro-tunnel docker stats --no-stream >> "$CPU_STATS_FILE"
}

rotate_tasks() {
    psql -h localhost -U postgres -d scheduler-kata \
        -c 'DELETE FROM tenant_default.queue'

    psql -h localhost -U postgres -d scheduler-kata \
        -c 'DELETE FROM tenant_default.task_bucket'

    psql -h localhost -U postgres -d scheduler-kata \
        -c "UPDATE tenant_default.task SET inactive_since = null, state = null WHERE state = 'failed' OR state = 'done'"

    psql -h localhost -U postgres -d scheduler-kata \
        -c "INSERT INTO tenant_default.task_bucket SELECT created_at, not_before, id
            FROM tenant_default.task WHERE tenant_default.task.state IS null"
}

time main "$@"
