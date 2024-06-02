-- inspired by https://github.com/tembo-io/pgmq
CREATE TABLE data.task (
    -- trick: reduce table size by rearranging columns: https://youtu.be/9_pbEVeMEB4?t=1082
    read_ct smallint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz DEFAULT NULL,
    vt timestamptz NOT NULL CONSTRAINT vt_check CHECK (vt >= now()) DEFAULT now(),
    id uuid PRIMARY KEY DEFAULT public.uuid_generate_v7 (),
    typ text NOT NULL CONSTRAINT typ_check CHECK (typ IN ('type_a', 'type_b', 'type_c')),
    state text DEFAULT NULL CONSTRAINT state_check CHECK (state IN (NULL, 'deleted', 'done'))
)
WITH (
    fillfactor = 80
);

CREATE INDEX ON data.task (state);

CREATE INDEX ON data.task (vt ASC);

-- this prevents setting custom ids
CREATE FUNCTION data.t_override_uuid ()
    RETURNS TRIGGER
    AS $$
BEGIN
    IF NEW.id IS NOT NULL THEN
        NEW.id := public.ts_to_uuid_v7 (NEW.created_at);
    END IF;
    RETURN new;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER t_update_uuid_before_insert
    BEFORE INSERT ON data.task
    FOR EACH ROW
    EXECUTE FUNCTION data.t_override_uuid ();

