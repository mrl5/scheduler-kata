CREATE FUNCTION data.t_notify_new_tasks ()
    RETURNS TRIGGER
    AS $$
BEGIN
    PERFORM
        pg_notify('task.new', json_build_object()::text);
    RETURN NULL;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER t_notify_after_insert
    AFTER INSERT ON data.task
    FOR EACH STATEMENT
    EXECUTE FUNCTION data.t_notify_new_tasks ();

