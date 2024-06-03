-- https://www.postgresql.org/docs/current/pgstattuple.html
SELECT
    *
FROM
    pgstattuple ('pg_catalog.pg_proc');

