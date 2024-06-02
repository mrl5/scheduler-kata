-- inspiration https://postgres.fm/episodes/partitioning-by-ulid
-- ref https://gitlab.com/postgres-ai/postgresql-consulting/postgres-howtos/-/blob/main/0065_uuid_v7_and_partitioning_timescaledb.md#uuid-v7-and-partitioning-timescaledb
CREATE FUNCTION public.uuid_generate_v7 ()
    RETURNS uuid
    AS $$
    -- use random v4 uuid as starting point (which has the same variant we need)
    -- then overlay timestamp
    -- then set version 7 by flipping the 2 and 1 bit in the version 4 string
    SELECT
        encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid ())
                    PLACING substring(int8send(floor(extract(epoch FROM clock_timestamp()) * 1000)::bigint)
                    FROM 3)
                FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid;
$$
LANGUAGE SQL
VOLATILE;

CREATE FUNCTION public.ts_to_uuid_v7 (timestamptz)
    RETURNS uuid
    AS $$
    SELECT
        encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid ())
                    PLACING substring(int8send(floor(extract(epoch FROM $1) * 1000)::bigint)
                        FROM 3)
                    FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid;
$$
LANGUAGE SQL
VOLATILE;

CREATE FUNCTION public.uuid_v7_to_ts (uuid_v7 uuid)
    RETURNS timestamptz
    AS $$
    SELECT
        to_timestamp(('x' || substring(encode(uuid_send(uuid_v7), 'hex')
                FROM 1 FOR 12))::bit(48)::bigint / 1000.0)::timestamptz;
$$
LANGUAGE sql;

