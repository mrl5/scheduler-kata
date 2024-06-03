-- https://www.cybertec-postgresql.com/en/postgresql-alter-default-privileges-permissions-explained
-- https://supabase.com/blog/postgres-roles-and-privileges
-- https://postgrest.org/en/v12/references/auth.html
--
CREATE SCHEMA data;

CREATE SCHEMA api;

CREATE SCHEMA api_util;

CREATE SCHEMA worker;

CREATE ROLE data_ro_group nologin;

CREATE ROLE data_rw_group nologin;

CREATE ROLE worker_group nologin;

GRANT data_ro_group TO data_rw_group WITH INHERIT TRUE;

-- anonymous is PGRST_DB_ANON_ROLE
CREATE ROLE anonymous NOLOGIN;

-- authenticator is DB_USER
GRANT anonymous TO authenticator;

-- for now lets skip JWT and allow RW
GRANT data_rw_group TO anonymous WITH INHERIT TRUE;

GRANT worker_group TO anonymous WITH INHERIT TRUE;

-- skipping "FOR USER postgres" so that ADMIN_DB_USER is applied automatically
ALTER DEFAULT PRIVILEGES IN SCHEMA data GRANT
SELECT
, INSERT, UPDATE ON TABLES TO data_rw_group;

ALTER DEFAULT PRIVILEGES IN SCHEMA data GRANT
SELECT
    ON TABLES TO data_ro_group;

ALTER DEFAULT PRIVILEGES IN SCHEMA data GRANT
SELECT
, UPDATE ON TABLES TO worker_group;

GRANT USAGE ON SCHEMA api TO data_ro_group;

GRANT USAGE ON SCHEMA api_util TO data_ro_group;

GRANT USAGE ON SCHEMA worker TO worker_group;

