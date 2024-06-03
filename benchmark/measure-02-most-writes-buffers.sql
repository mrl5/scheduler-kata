-- queries that write the most to shared_buffers
-- from https://www.crunchydata.com/developers/playground/query-performance-analytics
SELECT
    query,
    shared_blks_dirtied
FROM
    pg_stat_statements
WHERE
    shared_blks_dirtied > 0
ORDER BY
    2 DESC
LIMIT 5;

