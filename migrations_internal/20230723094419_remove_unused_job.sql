-- remove old cron job
DO $$
DECLARE id bigint;
DECLARE res record;
BEGIN
    SELECT jobid INTO id FROM cron.job
    WHERE jobname = 'refresh task_state_cached for all tenants';

    SELECT cron.unschedule(id) INTO res;
END $$;
