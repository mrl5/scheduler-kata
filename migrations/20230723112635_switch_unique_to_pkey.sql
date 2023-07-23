-- switch unique to pkey
-- * https://www.cybertec-postgresql.com/en/primary-keys-vs-unique-constraints-in-postgresql/
-- * https://dba.stackexchange.com/a/103112
-- * https://stackoverflow.com/questions/26365713/how-to-turn-a-unique-constraint-into-a-primary-key-in-postgresql

-- DISCLAIMER sqlx always run migrations as a transaction but CREATE INDEX
-- CONCURRENTLY can't be run in transaction
--
-- if it would be a production system, I'd probably do this manually (using
-- method from refs) to avoid locks and then run "void" migration for consistency

ALTER TABLE task DROP CONSTRAINT task_id_created_at_key CASCADE;
ALTER TABLE task ADD PRIMARY KEY (id, created_at);

ALTER TABLE queue ADD CONSTRAINT queue_task_id_task_created_at_fkey
    FOREIGN KEY (task_id, task_created_at) REFERENCES task (id, created_at);
