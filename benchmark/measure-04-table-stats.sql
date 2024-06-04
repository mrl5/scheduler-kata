-- https://www.crunchydata.com/blog/postgres-performance-boost-hot-updates-and-fill-factor
SELECT
    relname AS table_name,
    seq_scan AS sequential_scans,
    idx_scan AS index_scans,
    n_tup_ins AS inserts,
    n_tup_upd AS updates,
    n_tup_hot_upd AS hot_updates
FROM
    pg_stat_user_tables
WHERE
    relname = 'task';

