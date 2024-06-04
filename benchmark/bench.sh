#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TOP_DIR="$(dirname "$SCRIPT_DIR")"

main() {
    export $(grep -v '^#' "$TOP_DIR"/.env | xargs)
    export PGPASSWORD=${ADMIN_DB_PASSWORD}

    execute_sql -f "$SCRIPT_DIR"/00-load_data.sql
    execute_sql -c CHECKPOINT

    bench "$SCRIPT_DIR"/01-create_task.sql | grep tps > "$SCRIPT_DIR"/01-create_task.log
    execute_sql -c CHECKPOINT

    bench "$SCRIPT_DIR"/02-dequeue.sql | grep tps > "$SCRIPT_DIR"/02-dequeue.log
    execute_sql -c CHECKPOINT

    bench "$SCRIPT_DIR"/01-create_task.sql | grep tps > "$SCRIPT_DIR"/03-create_task.log
    execute_sql -c CHECKPOINT

    bench "$SCRIPT_DIR"/04-dequeue_and_complete.sql | grep tps > "$SCRIPT_DIR"/04-dequeue_and_complete.log
    execute_sql -c CHECKPOINT

    execute_sql -f "$SCRIPT_DIR"/measure-01-time-consuming.sql > "$SCRIPT_DIR"/measure-01-time-consuming.log
    execute_sql -f "$SCRIPT_DIR"/measure-02-most-writes-buffers.sql > "$SCRIPT_DIR"/measure-02-most-writes-buffers.log
    execute_sql -c 'CREATE EXTENSION IF NOT EXISTS pgstattuple'
    execute_sql -x -f "$SCRIPT_DIR"/measure-03-table-bloat.sql > "$SCRIPT_DIR"/measure-03-table-bloat.log
    execute_sql -x -f "$SCRIPT_DIR"/measure-04-table-stats.sql > "$SCRIPT_DIR"/measure-04-table-stats.log
}

execute_sql() {
    psql -U ${ADMIN_DB_USER} -h "$SCRIPT_DIR"/db_socket -d ${DB_NAME} "$@"
}

bench() {
    local sql_file="$1"

    pgbench -n -f "$sql_file" -U ${ADMIN_DB_USER} -h "$SCRIPT_DIR"/db_socket -j 5 -c 10 -T 10 ${DB_NAME}
}

main "$@"
