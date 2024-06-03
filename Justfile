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

dev-tools:
    cargo install hurl sqlx-cli

lint:
    pg_format -i migrations/*.sql benchmark/*.sql

test:
    hurl --test tests/*

db-bootstrap:
    # internal schema
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${LOCAL_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CREATE SCHEMA internal AUTHORIZATION ${ADMIN_DB_USER};"

    # app user
    PGPASSWORD=${ADMIN_DB_PASSWORD} psql -h ${LOCAL_DB_HOST} -p ${DB_PORT} \
        -U ${ADMIN_DB_USER} -d ${DB_NAME} \
        -c "CREATE ROLE ${DB_USER} WITH NOINHERIT LOGIN NOCREATEDB NOCREATEROLE NOSUPERUSER PASSWORD '${DB_PASSWORD}';"

db-migrate:
    sqlx migrate run -D \
        "postgres://${ADMIN_DB_USER}:${ADMIN_DB_PASSWORD}@${LOCAL_DB_HOST}:${DB_PORT}/${DB_NAME}?sslmode=disable&options=-c search_path=internal"
