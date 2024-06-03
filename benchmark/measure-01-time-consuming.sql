-- most time consuming queries
-- from https://www.crunchydata.com/developers/playground/query-performance-analytics
SELECT
    d.datname,
    round(s.total_exec_time::numeric, 2) AS total_exec_time,
    s.calls,
    s.rows,
    round(s.total_exec_time::numeric / calls, 2) AS avg_time,
    round((100 * s.total_exec_time / sum(s.total_exec_time::numeric) OVER ())::numeric, 2) AS percentage_cpu,
    substring(s.query, 1, 500) AS short_query
FROM
    pg_stat_statements s
    JOIN pg_database d ON (s.dbid = d.oid)
WHERE
    s.query != 'CHECKPOINT'
ORDER BY
    percentage_cpu DESC
LIMIT 5;

