set dotenv-load
set export

DOCKER_COMPOSE := "docker compose"

build:
    ${DOCKER_COMPOSE} build

run:
    ${DOCKER_COMPOSE} up

db-only:
    ${DOCKER_COMPOSE} up db

api-only:
    ${DOCKER_COMPOSE} up api

local-worker:
    DATABASE_URL=${DB_URI} cargo run --package worker

dev-tools:
    cargo install hurl sqlx-cli

lint:
    pg_format -i migrations/*.sql benchmark/*.sql worker/sql/*.sql
    rustfmt worker/src/*.rs
    SQLX_OFFLINE=true cargo clippy --fix --allow-staged --allow-dirty

test:
    hurl --test tests/*

db-bootstrap:
    # internal schema
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${LOCAL_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CREATE SCHEMA internal AUTHORIZATION ${ADMIN_DB_USER};"

    # api user
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${LOCAL_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CREATE ROLE ${DB_USER} WITH NOINHERIT LOGIN NOCREATEDB NOCREATEROLE NOSUPERUSER PASSWORD '${DB_PASSWORD}';"

    # worker user
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${LOCAL_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CREATE ROLE ${WORKER_DB_USER} WITH NOINHERIT LOGIN NOCREATEDB NOCREATEROLE NOSUPERUSER PASSWORD '${WORKER_DB_PASSWORD}';"

db-migrate:
    sqlx migrate run -D \
        "postgres://${ADMIN_DB_USER}:${ADMIN_DB_PASSWORD}@${LOCAL_DB_HOST}:${DB_PORT}/${DB_NAME}?sslmode=disable&options=-c search_path=internal"

sqlx-prepare:
    cargo sqlx prepare --workspace -D "${DB_URI}"
