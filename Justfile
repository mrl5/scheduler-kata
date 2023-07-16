set dotenv-load
set export

TMRW := `date -u -Iseconds -d"+1days"`
DOCKER_COMPOSE := "docker compose"
TENANT := env_var("TENANT")

build:
    ${DOCKER_COMPOSE} build

run:
    ${DOCKER_COMPOSE} up

db-only:
    ${DOCKER_COMPOSE} up db

dev-tools:
    cargo install hurl sqlx-cli

local-app:
    cargo run --package app-monolith

local-api:
    cargo run --package app-rest-api

local-scheduler:
    cargo run --package scheduler

local-worker:
    cargo run --package worker

test: test-unit test-api

test-unit:
    SQLX_OFFLINE=true cargo test

test-api:
    hurl --test --variable tomorrow={{TMRW}} ./tests/*.hurl

lint: fmt
    SQLX_OFFLINE=true cargo clippy --fix --allow-staged

fmt:
    rustfmt crates/**/src/*.rs

db-bootstrap:
    # https://www.crunchydata.com/blog/logging-tips-for-postgres-featuring-your-slow-queries
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "ALTER DATABASE \"${DB_NAME}\" SET log_min_duration_statement = '100ms';"

    # https://www.crunchydata.com/blog/control-runaway-postgres-queries-with-statement-timeout
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "ALTER DATABASE \"${DB_NAME}\" SET statement_timeout = '60s';"

    # https://docs.crunchybridge.com/extensions-and-languages/auto_explain
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "ALTER SYSTEM SET session_preload_libraries = 'auto_explain';"
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "SELECT pg_reload_conf();"
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "ALTER SYSTEM SET auto_explain.log_min_duration = 2000;"
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "ALTER SYSTEM SET auto_explain.log_analyze = on;"
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "ALTER SYSTEM SET auto_explain.log_triggers = on;"
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "ALTER SYSTEM SET auto_explain.log_nested_statements = on;"
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "SELECT pg_reload_conf();"

    # allow using pg_cron in our db
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
    -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "ALTER SYSTEM SET cron.database_name TO '${DB_NAME}';"

    # restart db so that changes are effective
    ${DOCKER_COMPOSE} restart db
    sleep 5

    # internal schema
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CREATE SCHEMA internal AUTHORIZATION ${ADMIN_DB_USER};"

    cat ./migrations_internal/*.sql \
    | PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME}

db-add-new-tenant:
    just --dotenv-filename .env.sqlx _db-add-new-tenant

_db-add-new-tenant:
    # workaround sqlx limitation
    # https://github.com/launchbadge/sqlx/issues/1835#issuecomment-1493727747
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CREATE USER ${DB_USER} WITH PASSWORD '${DB_PASSWORD}';"

    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CALL internal.create_new_tenant('${ADMIN_DB_USER}', '${TENANT}');"

db-migrate:
    just --dotenv-filename .env.sqlx TENANT=${TENANT} _db-migrate

_db-migrate:
    sqlx migrate run -D "${DB_URI}&options=-c search_path=tenant_${TENANT}"
    cargo sqlx prepare --workspace -D "${DB_URI}&options=-c search_path=tenant_${TENANT}"

    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CALL internal.create_partition_for_now('${TENANT}', 'task');"

    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${SQLX_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CALL internal.create_partition_for_next_month('${TENANT}', 'task');"
